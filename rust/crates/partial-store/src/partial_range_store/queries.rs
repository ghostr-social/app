//! The read-only half of the store. Every lookup marks the key as
//! recently used, which is what keeps the video being watched at the
//! safe end of the eviction order.

use crate::partial_range_paths::validate_key;
use crate::partial_range_store::capacity::CapacitySnapshot;
use crate::partial_range_store::single_response::SingleResponseStorage;
use crate::partial_range_store::PartialRangeStore;
use anyhow::Result;
use std::ops::Range;

mod evidence;
mod provisional;
mod read;
mod snapshot;

pub use evidence::StoredEvidenceId;
pub use snapshot::StoredMediaSnapshot;

#[cfg(test)]
mod tests;

impl PartialRangeStore {
    /// One generation-coherent observation for planning and projection.
    pub async fn media_snapshot(&self, key: &str) -> Result<StoredMediaSnapshot> {
        validate_key(key)?;
        let _update = self.observe_key(key).await?;
        let provisional = provisional::capture(self, key).await;
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        let total_len = entry.manifest.total_len();
        let stable_ranges = entry.manifest.ranges();
        let readable = provisional::merge(&entry.manifest, &provisional)?;
        let ranges = readable.ranges();
        let evidence = evidence::StoredEvidence::capture(&readable);
        let finalized = entry.completion.is_some();
        let complete = finalized || entry.manifest.is_complete();
        let binding = self.representations.lock().await.get(key).cloned();
        let revision = self.current_content_revision(key).await;
        let response_storage = self
            .single_response_actions
            .lock()
            .await
            .get(key)
            .map(|state| state.storage);
        let planning_ranges = match response_storage {
            Some(SingleResponseStorage::Live { .. }) => Vec::new(),
            _ => stable_ranges,
        };
        let continuation_source = if planning_ranges.is_empty() || complete {
            None
        } else {
            self.source_generations
                .lock()
                .await
                .get(key)
                .map(|(source, _)| source.clone())
        };
        Ok(StoredMediaSnapshot {
            binding,
            revision,
            total_len,
            ranges,
            planning_ranges,
            complete,
            finalized,
            continuation_source,
            evidence,
        })
    }

    /// Effective storage limit and total accounted usage from one
    /// write-serialized observation.
    pub async fn capacity_snapshot(&self) -> CapacitySnapshot {
        let _entries = self.entries.lock().await;
        let used = *self.used_bytes.lock().await;
        let limit = self.capacity.cap(&self.root, used).await;
        let revision = self.capacity.events().revision();
        CapacitySnapshot::new(limit, used, revision)
    }

    pub async fn used_bytes(&self) -> u64 {
        *self.used_bytes.lock().await
    }

    pub async fn total_len(&self, key: &str) -> Result<Option<u64>> {
        let _update = self.observe_key(key).await?;
        let mut entries = self.entries.lock().await;
        Ok(self.entry(&mut entries, key).await?.manifest.total_len())
    }

    pub async fn present_ranges(&self, key: &str) -> Result<Vec<Range<u64>>> {
        let _update = self.observe_key(key).await?;
        let provisional = provisional::capture(self, key).await;
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        Ok(provisional::merge(&entry.manifest, &provisional)?.ranges())
    }

    pub async fn missing_within(&self, key: &str, span: Range<u64>) -> Result<Vec<Range<u64>>> {
        let _update = self.observe_key(key).await?;
        let provisional = provisional::capture(self, key).await;
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        Ok(provisional::merge(&entry.manifest, &provisional)?.missing_within(&span))
    }

    pub async fn read_range(&self, key: &str, span: Range<u64>) -> Result<Option<Vec<u8>>> {
        let _update = self.observe_key(key).await?;
        let provisional = provisional::capture(self, key).await;
        let plan = {
            let mut entries = self.entries.lock().await;
            let touched = self.tick();
            let entry = self.entry(&mut entries, key).await?;
            let readable = provisional::merge(&entry.manifest, &provisional)?;
            let plan =
                read::ReadPlan::capture_with_manifest(&self.paths, key, entry, &readable, span)?;
            if plan.is_some() {
                entry.touched = touched;
            }
            plan
        };
        let Some(plan) = plan else { return Ok(None) };
        let outcome = plan.execute().await;
        self.finish_read(key, plan, outcome).await
    }

    async fn finish_read(
        &self,
        key: &str,
        plan: read::ReadPlan,
        outcome: Result<read::ReadOutcome>,
    ) -> Result<Option<Vec<u8>>> {
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        if !plan.is_current(&self.paths, key, entry)? {
            return Ok(None);
        }
        if let Some(bytes) = read::verified_bytes(outcome) {
            return Ok(Some(bytes));
        }
        match read::classify_retry(plan.execute().await) {
            read::RetryOutcome::Verified(bytes) => return Ok(Some(bytes)),
            read::RetryOutcome::Transient(error) => return Err(error),
            read::RetryOutcome::StructuralLoss => {}
        }
        self.discard(&mut entries, key).await?;
        Ok(None)
    }

    pub async fn is_complete(&self, key: &str) -> Result<bool> {
        let _update = self.observe_key(key).await?;
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        Ok(entry.completion.is_some() || entry.manifest.is_complete())
    }
}
