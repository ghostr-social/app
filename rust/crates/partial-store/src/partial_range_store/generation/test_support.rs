use super::PartialRangeStore;
use anyhow::{bail, Result};
use ghostr_engine::representation::{SourceGeneration, TransferIdentity};

impl PartialRangeStore {
    /// Selects the source generation used by legacy test scenarios.
    ///
    /// # Errors
    ///
    /// Returns an error when the stored binding cannot be read or replaced.
    pub async fn select_transfer(
        &self,
        identity: TransferIdentity,
    ) -> Result<Option<SourceGeneration>> {
        let _update = self.update_key(identity.post().as_str()).await?;
        let binding = self.current_binding(&identity).await?;
        let key = identity.post().as_str().to_owned();
        let already_selected = self.selected().get(&key) == Some(&identity);
        self.restore_compatible_generation(&binding).await?;
        let restored = self.generation_for(&key, identity.source().as_str()).await;
        if already_selected {
            return Ok(restored);
        }
        self.selected().remove(&key);
        self.cancel_single_response(&key).await?;
        if restored.is_none() && !self.http_generation_preserves_bytes(&identity).await {
            self.replace_stored_generation(&key, &binding, None).await?;
        }
        self.selected().insert(key, identity);
        Ok(restored)
    }

    /// Associates a validated generation with the selected test transfer.
    ///
    /// # Errors
    ///
    /// Returns an error when the transfer is stale or persistence fails.
    pub async fn accept_generation(
        &self,
        identity: &TransferIdentity,
        generation: SourceGeneration,
    ) -> Result<()> {
        let _update = self.update_key(identity.post().as_str()).await?;
        self.current_binding(identity).await?;
        let key = identity.post().as_str();
        self.ensure_sparse_mutable(key).await?;
        if self.selected().get(key) != Some(identity) {
            bail!("sparse transfer is no longer selected");
        }
        self.accept_generation_locked(identity, generation).await
    }

    /// Writes test bytes only while the generation remains authoritative.
    ///
    /// # Errors
    ///
    /// Returns an error when store synchronization or persistence fails.
    pub async fn write_range_for_generation_if_current(
        &self,
        identity: &TransferIdentity,
        generation: &SourceGeneration,
        offset: u64,
        bytes: &[u8],
    ) -> Result<bool> {
        let _update = self.update_key(identity.post().as_str()).await?;
        if !self.generation_is_current(identity, generation).await {
            return Ok(false);
        }
        let mut entries = self.entries.lock().await;
        if !self.generation_is_current(identity, generation).await {
            return Ok(false);
        }
        self.write_range_locked(&mut entries, identity.post().as_str(), offset, bytes)
            .await?;
        Ok(self.generation_is_current(identity, generation).await)
    }
}
