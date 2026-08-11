use super::PartialRangeStore;
use crate::partial_range_paths::validate_key;
use crate::partial_range_representation_disk as identity_disk;
use anyhow::Result;
use ghostr_engine::representation::{RepresentationBinding, TransferIdentity};
use std::sync::MutexGuard;

impl PartialRangeStore {
    pub async fn bind_representation(&self, binding: RepresentationBinding) -> Result<()> {
        let key = binding.post().as_str();
        validate_key(key)?;
        let _update = self.representation_updates.lock().await;
        if self.binding_is_current(&binding).await {
            return Ok(());
        }
        self.install_binding(binding.clone()).await;
        if let Err(error) = self.bind_stored_bytes(&binding).await {
            self.remove_binding_if(&binding).await;
            return Err(error);
        }
        Ok(())
    }

    pub async fn write_range_for_transfer_if_current(
        &self,
        identity: &TransferIdentity,
        offset: u64,
        bytes: &[u8],
    ) -> Result<bool> {
        if !self.transfer_is_current(identity).await {
            return Ok(false);
        }
        let mut entries = self.entries.lock().await;
        if !self.transfer_is_current(identity).await {
            return Ok(false);
        }
        self.write_range_locked(&mut entries, identity.post().as_str(), offset, bytes)
            .await?;
        if self.transfer_is_current(identity).await {
            return Ok(true);
        }
        self.discard_stale_write(&mut entries, identity.post().as_str())
            .await?;
        Ok(false)
    }

    pub fn select_transfer(&self, identity: TransferIdentity) {
        self.selected()
            .insert(identity.post().as_str().to_owned(), identity);
    }

    pub(super) async fn clear_representation_bindings(&self) {
        self.representations.lock().await.clear();
        self.selected().clear();
    }

    async fn bind_stored_bytes(&self, binding: &RepresentationBinding) -> Result<()> {
        let key = binding.post().as_str();
        let path = self.paths.representation(key);
        let stored = identity_disk::load(&path).await?;
        if stored.as_deref() == Some(binding.representation().fingerprint()) {
            return Ok(());
        }
        let mut entries = self.entries.lock().await;
        self.discard(&mut entries, key).await?;
        identity_disk::save(&path, binding.representation().fingerprint()).await
    }

    async fn discard_stale_write(&self, entries: &mut super::Entries, key: &str) -> Result<()> {
        self.discard(entries, key).await?;
        let binding = self.representations.lock().await.get(key).cloned();
        if let Some(binding) = binding {
            identity_disk::save(
                &self.paths.representation(key),
                binding.representation().fingerprint(),
            )
            .await?;
        }
        Ok(())
    }

    async fn binding_is_current(&self, binding: &RepresentationBinding) -> bool {
        self.representations
            .lock()
            .await
            .get(binding.post().as_str())
            == Some(binding)
    }

    async fn install_binding(&self, binding: RepresentationBinding) {
        self.selected().remove(binding.post().as_str());
        self.representations
            .lock()
            .await
            .insert(binding.post().as_str().to_owned(), binding);
    }

    async fn remove_binding_if(&self, binding: &RepresentationBinding) {
        let mut bindings = self.representations.lock().await;
        if bindings.get(binding.post().as_str()) == Some(binding) {
            bindings.remove(binding.post().as_str());
        }
    }

    pub async fn transfer_is_current(&self, identity: &TransferIdentity) -> bool {
        let bound = self
            .representations
            .lock()
            .await
            .get(identity.post().as_str())
            .and_then(|binding| binding.transfer(identity.source().as_str()))
            .as_ref()
            == Some(identity);
        bound && self.selected().get(identity.post().as_str()) == Some(identity)
    }

    fn selected(&self) -> MutexGuard<'_, std::collections::HashMap<String, TransferIdentity>> {
        self.selected_transfers
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }
}
