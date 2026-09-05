use super::PartialRangeStore;
use anyhow::Result;

impl PartialRangeStore {
    /// Revokes session writes and read authority, discarding volatile media only.
    /// Public disk bytes remain available for fresh session authorization.
    ///
    /// # Errors
    /// Returns an error if a volatile object's obsolete disk metadata cannot be removed.
    pub async fn reset_playback_access(&self) -> Result<()> {
        let _update = self.representation_updates.write().await;
        self.revoke_all_actions().await;
        self.clear_representation_bindings().await;
        let keys = self
            .transient_responses
            .lock()
            .await
            .keys()
            .cloned()
            .collect();
        let mut entries = self.entries.lock().await;
        let result = self.discard_keys(&mut entries, keys).await;
        self.changed.notify_waiters();
        result
    }
}
