//! The read-only half of the store. Every lookup marks the key as
//! recently used, which is what keeps the video being watched at the
//! safe end of the eviction order.

use crate::partial_range_disk as disk;
use crate::partial_range_store::PartialRangeStore;
use anyhow::Result;
use std::ops::Range;
use std::path::PathBuf;

impl PartialRangeStore {
    pub async fn used_bytes(&self) -> u64 {
        *self.used_bytes.lock().await
    }

    pub async fn total_len(&self, key: &str) -> Result<Option<u64>> {
        let mut entries = self.entries.lock().await;
        Ok(self.entry(&mut entries, key).await?.manifest.total_len())
    }

    pub async fn present_ranges(&self, key: &str) -> Result<Vec<Range<u64>>> {
        let mut entries = self.entries.lock().await;
        Ok(self.entry(&mut entries, key).await?.manifest.ranges())
    }

    pub async fn missing_within(&self, key: &str, span: Range<u64>) -> Result<Vec<Range<u64>>> {
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        Ok(entry.manifest.missing_within(&span))
    }

    pub async fn read_range(&self, key: &str, span: Range<u64>) -> Result<Option<Vec<u8>>> {
        let mut entries = self.entries.lock().await;
        let Some(path) = self.readable_path(&mut entries, key, &span).await? else {
            return Ok(None);
        };
        disk::read_span(&path, &span).await.map(Some)
    }

    pub async fn is_complete(&self, key: &str) -> Result<bool> {
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        Ok(entry.completion.is_some() || entry.manifest.is_complete())
    }

    /// The file holding `span`, or `None` when the store does not have
    /// every byte of it yet. Reading counts as use.
    async fn readable_path(
        &self,
        entries: &mut super::Entries,
        key: &str,
        span: &Range<u64>,
    ) -> Result<Option<PathBuf>> {
        let touched = self.tick();
        let entry = self.entry(entries, key).await?;
        if !entry.manifest.contains(span) {
            return Ok(None);
        }
        entry.touched = touched;
        Ok(Some(match entry.completion {
            Some(_) => self.paths.completed(key),
            None => self.paths.partial(key),
        }))
    }
}
