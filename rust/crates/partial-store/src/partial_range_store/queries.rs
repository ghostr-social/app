//! The read-only half of the store. Every lookup marks the key as
//! recently used, which is what keeps the video being watched at the
//! safe end of the eviction order.

use crate::partial_range_store::capacity::CapacitySnapshot;
use crate::partial_range_store::PartialRangeStore;
use anyhow::Result;
use core::ops::Range;

mod evidence;
mod media_snapshot;
mod provisional;
mod read;
mod session;
mod snapshot;

pub use evidence::StoredEvidenceId;
pub use snapshot::StoredMediaSnapshot;

#[cfg(test)]
mod tests;

impl PartialRangeStore {
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

    /// # Errors
    ///
    /// Returns an error when the key or its persisted manifest cannot be read.
    pub async fn total_len(&self, key: &str) -> Result<Option<u64>> {
        let _update = self.observe_key(key).await?;
        if let Some(response) = self.session_response(key).await {
            return Ok(response.manifest().total_len());
        }
        let mut entries = self.entries.lock().await;
        Ok(self.entry(&mut entries, key).await?.manifest.total_len())
    }

    /// # Errors
    ///
    /// Returns an error when the key or its coherent range state cannot be read.
    pub async fn present_ranges(&self, key: &str) -> Result<Vec<Range<u64>>> {
        let _update = self.observe_key(key).await?;
        if let Some(response) = self.session_response(key).await {
            return Ok(response.manifest().ranges());
        }
        let provisional = provisional::capture(self, key).await;
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        Ok(provisional::merge(&entry.manifest, &provisional)?.ranges())
    }

    /// # Errors
    ///
    /// Returns an error when the key or its coherent range state cannot be read.
    pub async fn missing_within(&self, key: &str, span: Range<u64>) -> Result<Vec<Range<u64>>> {
        let _update = self.observe_key(key).await?;
        if let Some(response) = self.session_response(key).await {
            return Ok(response.manifest().missing_within(&span));
        }
        let provisional = provisional::capture(self, key).await;
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        Ok(provisional::merge(&entry.manifest, &provisional)?.missing_within(&span))
    }

    /// # Errors
    ///
    /// Returns an error when persisted bytes cannot be read or fail integrity validation.
    pub async fn read_range(&self, key: &str, span: Range<u64>) -> Result<Option<Vec<u8>>> {
        let _update = self.observe_key(key).await?;
        if let Some(response) = self.session_response(key).await {
            return session::read(self, key, &response, span).await;
        }
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
        if !plan.is_current(&self.paths, key, entry) {
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

    /// # Errors
    ///
    /// Returns an error when the key or its persisted manifest cannot be read.
    pub async fn is_complete(&self, key: &str) -> Result<bool> {
        let _update = self.observe_key(key).await?;
        if self.session_response(key).await.is_some() {
            return Ok(true);
        }
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        Ok(entry.completion.is_some() || entry.manifest.is_complete())
    }
}
