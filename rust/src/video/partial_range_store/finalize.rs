//! Leaving the partial pool: a byte-complete file is judged, then
//! either promoted to its completed name or discarded.

use crate::video::partial_range_completion::{self as completion, Completion};
use crate::video::partial_range_disk as disk;
use crate::video::partial_range_store::{Entries, PartialRangeStore};
use anyhow::{bail, Context, Result};
use std::path::PathBuf;

impl PartialRangeStore {
    /// Moves a byte-complete file out of the partial pool. `advertised`
    /// is the note's `imeta x` when it has one: absent, the bytes are
    /// kept unverified; present, they must hash to it or they are lost.
    pub async fn finalize(&self, key: &str, advertised: Option<&str>) -> Result<PathBuf> {
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        if entry.completion.is_some() {
            return Ok(self.paths.completed(key));
        }
        if !entry.manifest.is_complete() {
            bail!("cannot finalize a video with missing ranges");
        }
        match completion::judge(&self.paths.partial(key), advertised).await? {
            Some(verdict) => self.promote(&mut entries, key, verdict).await,
            None => {
                self.discard(&mut entries, key).await?;
                bail!("partial video digest does not match the expected digest")
            }
        }
    }

    async fn promote(
        &self,
        entries: &mut Entries,
        key: &str,
        verdict: Completion,
    ) -> Result<PathBuf> {
        let completed = self.paths.completed(key);
        tokio::fs::rename(&self.paths.partial(key), &completed)
            .await
            .context("promote complete partial video")?;
        disk::remove_if_present(&self.paths.manifest(key)).await?;
        completion::record(&self.paths.verified(key), verdict).await?;
        let entry = entries.get_mut(key).context("promoted entry present")?;
        entry.completion = Some(verdict);
        self.changed.notify_waiters();
        Ok(completed)
    }
}
