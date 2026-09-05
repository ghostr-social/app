use super::{cleanup_debt, policy_intent, Entries, PartialRangeStore};
use crate::partial_range_disk as disk;
use crate::partial_range_manifest::RangeManifest;
use anyhow::{Context as _, Error, Result};

impl PartialRangeStore {
    pub(super) async fn discard(&self, entries: &mut Entries, key: &str) -> Result<()> {
        self.discard_with_durability(entries, key, false).await
    }

    pub(super) async fn discard_before_authority(
        &self,
        entries: &mut Entries,
        key: &str,
    ) -> Result<()> {
        self.discard_with_durability(entries, key, true).await
    }

    async fn discard_with_durability(
        &self,
        entries: &mut Entries,
        key: &str,
        durable: bool,
    ) -> Result<()> {
        self.advance_content_revision(key).await;
        self.transient_responses.lock().await.remove(key);
        self.source_generations.lock().await.remove(key);
        self.http_generations.lock().await.remove(key);
        self.selected().remove(key);
        self.changed.notify_waiters();
        let pending = self
            .take_sparse_response_bytes(key)
            .await
            .saturating_add(self.take_session_response(key).await);
        let response_error = self.cancel_single_response(key).await.err();
        let removal_error = self.remove_paths(key).await;
        let durability_error = if response_error.is_none() && removal_error.is_none() && durable {
            self.sync_removed_paths(key).await.err()
        } else {
            None
        };
        if let Some(error) = response_error.or(removal_error).or(durability_error) {
            self.mark_quarantined(entries, key);
            self.transfer_charged_cleanup_debt(
                key,
                cleanup_debt::CleanupScope::CanonicalDirty,
                pending,
            )
            .await?;
            return Err(error);
        }
        if let Some(entry) = entries.remove(key) {
            self.release(entry.accounted).await;
        }
        self.release(pending).await;
        self.release_cleanup_debts(key).await;
        Ok(())
    }

    pub(super) async fn take_sparse_response_bytes(&self, key: &str) -> u64 {
        let mut responses = self.sparse_response_actions.lock().await;
        let owned: Vec<_> = responses
            .iter()
            .filter(|(_, state)| state.identity.post().as_str() == key)
            .map(|(id, _)| *id)
            .collect();
        owned
            .into_iter()
            .filter_map(|id| responses.remove(&id))
            .map(|state| {
                state.owner.revoke();
                state.pending
            })
            .sum()
    }

    async fn remove_paths(&self, key: &str) -> Option<Error> {
        let mut failure = None;
        for path in self.paths.all(key) {
            if let Err(error) = disk::remove_if_present(&path).await {
                failure.get_or_insert(error);
            }
        }
        if failure.is_none() {
            if let Err(error) = policy_intent::remove_authority(&self.paths, key).await {
                failure = Some(error);
            }
        }
        failure
    }

    async fn sync_removed_paths(&self, key: &str) -> Result<()> {
        tokio::fs::create_dir_all(&self.root)
            .await
            .context("create partial store directory")?;
        disk::sync_parent(&self.paths.partial(key)).await
    }

    fn mark_quarantined(&self, entries: &mut Entries, key: &str) {
        if let Some(entry) = entries.get_mut(key) {
            entry.manifest = RangeManifest::default();
            entry.completion = None;
            entry.touched = self.tick();
        }
    }
}

#[cfg(any(test, feature = "test"))]
mod test_support;
