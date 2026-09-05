//! Explicit teardown of every progressive artifact owned by the store.

use super::reload::stored_keys;
use super::PartialRangeStore;
use anyhow::Result;
use core::sync::atomic::Ordering;
use std::collections::BTreeSet;

impl PartialRangeStore {
    /// Removes every progressive artifact and resets store accounting.
    ///
    /// # Errors
    ///
    /// Returns an error when persisted store artifacts cannot be enumerated or removed.
    pub async fn clear(&self) -> Result<()> {
        let _update = self.representation_updates.write().await;
        self.revoke_all_actions().await;
        self.clear_representation_bindings().await;
        let mut keys = stored_keys(&self.root).await?;
        let mut entries = self.entries.lock().await;
        keys.extend(entries.keys().cloned());
        let result = self.discard_keys(&mut entries, keys).await;
        self.sparse_response_actions.lock().await.clear();
        *self.refused.lock().await = None;
        self.refusals.store(0, Ordering::Relaxed);
        self.capacity.events().signal();
        self.changed.notify_waiters();
        result
    }

    pub(super) async fn discard_keys(
        &self,
        entries: &mut super::Entries,
        keys: BTreeSet<String>,
    ) -> Result<()> {
        let mut failure = None;
        for key in keys {
            if let Err(error) = self.discard(entries, &key).await {
                failure.get_or_insert(error);
            }
        }
        failure.map_or(Ok(()), Err)
    }
}
