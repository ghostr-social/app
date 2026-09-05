use super::{PartialRangeStore, Result};

impl PartialRangeStore {
    /// Invalidates one object's cached authority and removes all of its bytes.
    ///
    /// # Errors
    ///
    /// Returns an error when the object state cannot be loaded or removed durably.
    pub async fn quarantine(&self, key: &str) -> Result<()> {
        let _update = self.update_key(key).await?;
        let mut entries = self.entries.lock().await;
        self.entry(&mut entries, key).await?;
        self.discard(&mut entries, key).await
    }
}
