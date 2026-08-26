use super::{EvictionOutcome, PartialRangeStore};
use anyhow::Result;
use core::ops::Range;

impl PartialRangeStore {
    /// Applies an eviction without requiring a production policy revision.
    ///
    /// # Errors
    ///
    /// Returns an error when the eviction transaction cannot be committed.
    pub async fn evict_ranges(&self, key: &str, ranges: &[Range<u64>]) -> Result<EvictionOutcome> {
        self.evict_ranges_at_revision(key, ranges, None).await
    }
}
