//! Explicit teardown of every progressive artifact owned by the store.

use super::reload::stored_keys;
use super::PartialRangeStore;
use anyhow::Result;
use std::collections::BTreeSet;
use std::sync::atomic::Ordering;

impl PartialRangeStore {
    pub async fn clear(&self) -> Result<()> {
        let _update = self.representation_updates.lock().await;
        self.clear_representation_bindings().await;
        let mut keys = stored_keys(&self.root).await;
        let mut entries = self.entries.lock().await;
        keys.extend(entries.keys().cloned());
        self.discard_keys(&mut entries, keys).await?;
        *self.refused.lock().await = None;
        self.refusals.store(0, Ordering::Relaxed);
        self.capacity.events().signal();
        self.changed.notify_waiters();
        Ok(())
    }

    async fn discard_keys(
        &self,
        entries: &mut super::Entries,
        keys: BTreeSet<String>,
    ) -> Result<()> {
        for key in keys {
            self.discard(entries, &key).await?;
        }
        Ok(())
    }
}
