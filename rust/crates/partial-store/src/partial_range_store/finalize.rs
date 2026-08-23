//! Leaving the partial pool: a byte-complete file is judged, then
//! either promoted to its completed name or discarded.

use crate::partial_range_completion::{self as completion, Completion, IntegrityMismatch};
use crate::partial_range_disk as disk;
use crate::partial_range_store::{Entries, PartialRangeStore};
use anyhow::{bail, Context, Result};

mod session;

impl PartialRangeStore {
    /// Moves a byte-complete file out of the partial pool. `advertised`
    /// is the note's `imeta x` when it has one: absent, the bytes are
    /// kept unverified; present, they must hash to it or they are lost.
    pub async fn finalize(&self, key: &str, advertised: Option<&str>) -> Result<Completion> {
        let _update = self.update_key(key).await?;
        let mut entries = self.entries.lock().await;
        if let Some(response) = self.session_response(key).await {
            return session::finalize(self, &mut entries, key, advertised, &response).await;
        }
        let entry = self.entry(&mut entries, key).await?;
        if let Some(completion) = entry.completion {
            self.retire_generation(key).await;
            if completion == Completion::Verified {
                self.retire_http_generation(key).await;
            }
            return Ok(completion);
        }
        if !entry.manifest.is_complete() {
            bail!("cannot finalize a video with missing ranges");
        }
        match completion::judge(&self.paths.partial(key), advertised).await? {
            Some(verdict) => self.promote(&mut entries, key, verdict).await,
            None => {
                self.discard(&mut entries, key).await?;
                Err(IntegrityMismatch.into())
            }
        }
    }

    async fn promote(
        &self,
        entries: &mut Entries,
        key: &str,
        verdict: Completion,
    ) -> Result<Completion> {
        let completed = self.paths.completed(key);
        completion::record(&self.paths.verified(key), verdict).await?;
        tokio::fs::rename(&self.paths.partial(key), &completed)
            .await
            .context("promote complete partial video")?;
        disk::sync_parent(&completed).await?;
        let entry = entries.get_mut(key).context("promoted entry present")?;
        entry.completion = Some(verdict);
        self.retire_generation(key).await;
        if verdict == Completion::Verified {
            self.retire_http_generation(key).await;
        }
        self.changed.notify_waiters();
        Ok(verdict)
    }
}
