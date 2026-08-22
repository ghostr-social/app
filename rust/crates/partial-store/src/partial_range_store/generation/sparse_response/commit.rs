use super::SparseResponseState;
use crate::partial_range_disk as disk;
use crate::partial_range_store::PartialRangeStore;
use anyhow::{ensure, Result};
use sha2::Digest;
use std::ops::Range;

struct CommitRecord {
    span: Range<u64>,
    checksum: String,
}

impl PartialRangeStore {
    pub(super) async fn commit_sparse_response(
        &self,
        key: &str,
        state: SparseResponseState,
    ) -> Result<bool> {
        disk::sync_file(&self.paths.partial(key)).await?;
        let record = self.sparse_commit_record(key, &state).await?;
        let Some(record) = record else {
            return self.discard_sparse_response(key).await;
        };
        let accounted = self.publish_sparse_response(key, &state, record).await?;
        self.finish_sparse_intent_with_total(key, &state, accounted)
            .await?;
        self.remove_sparse_state(&state.owner).await;
        self.changed.notify_waiters();
        Ok(true)
    }

    async fn sparse_commit_record(
        &self,
        key: &str,
        state: &SparseResponseState,
    ) -> Result<Option<CommitRecord>> {
        let span = state.range.start..state.next_offset;
        let checksum = format!("{:x}", state.hasher.clone().finalize());
        let observed = disk::sha256_span(&self.paths.partial(key), &span).await?;
        Ok((observed == checksum).then_some(CommitRecord { span, checksum }))
    }

    async fn publish_sparse_response(
        &self,
        key: &str,
        state: &SparseResponseState,
        record: CommitRecord,
    ) -> Result<u64> {
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        let mut manifest = entry.manifest.clone();
        ensure!(
            manifest.missing_within(&record.span) == [record.span.clone()],
            "sparse action no longer owns a hole"
        );
        manifest.insert(record.span.clone())?;
        manifest.record_checksum(record.span, record.checksum)?;
        disk::save_manifest(&self.paths.manifest(key), &manifest).await?;
        entry.manifest = manifest;
        entry.accounted = entry.accounted.saturating_add(state.received);
        entry.touched = self.tick();
        self.mark_sparse_committed(&state.owner, state.received)
            .await?;
        Ok(entry.accounted)
    }
}
