use super::PartialRangeStore;
use anyhow::{ensure, Context as _, Result};
use sha2::Digest as _;

impl PartialRangeStore {
    pub(in crate::partial_range_store) async fn write_transient(
        &self,
        key: &str,
        offset: u64,
        bytes: &[u8],
    ) -> Result<bool> {
        let mut entries = self.entries.lock().await;
        let mut responses = self.transient_responses.lock().await;
        let response = responses.get_mut(key).context("transient response")?;
        let end = offset
            .checked_add(bytes.len() as u64)
            .context("transient offset")?;
        ensure!(
            offset == response.bytes.len() as u64 && !bytes.is_empty() && end <= response.limit,
            "invalid transient write"
        );
        response.bytes.extend_from_slice(bytes);
        response.digest.update(bytes);
        let entry = entries.get_mut(key).context("transient entry")?;
        entry.manifest.insert(offset..end)?;
        entry
            .manifest
            .record_checksum(0..end, format!("{:x}", response.digest.clone().finalize()))?;
        entry.touched = self.tick();
        self.changed.notify_waiters();
        Ok(true)
    }

    pub(in crate::partial_range_store) async fn finish_transient(
        &self,
        key: &str,
        total: u64,
    ) -> Result<bool> {
        let mut entries = self.entries.lock().await;
        let mut responses = self.transient_responses.lock().await;
        let response = responses.get_mut(key).context("transient response")?;
        ensure!(
            response.bytes.len() as u64 == total,
            "incomplete transient body"
        );
        entries
            .get_mut(key)
            .context("transient entry")?
            .manifest
            .set_total_len(total)?;
        response.complete = true;
        self.changed.notify_waiters();
        Ok(true)
    }

    pub(in crate::partial_range_store) async fn abort_transient(&self, key: &str) -> Result<()> {
        let removed = self.transient_responses.lock().await.remove(key);
        if removed.is_some() {
            let mut entries = self.entries.lock().await;
            if let Some(entry) = entries.get_mut(key) {
                entry.manifest = crate::partial_range_manifest::RangeManifest::default();
                entry.completion = None;
            }
        }
        // The caller owns the key update. Canonical bytes were removed on open.
        self.advance_content_revision(key).await;
        self.changed.notify_waiters();
        Ok(())
    }
}
