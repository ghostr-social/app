use super::PartialRangeStore;
use crate::partial_range_generation_disk::{self as disk, StoredGeneration};
use crate::partial_range_representation_disk as representation_disk;
use anyhow::{bail, Result};
use ghostr_engine::representation::{SourceGeneration, TransferIdentity};

impl PartialRangeStore {
    pub async fn select_transfer(
        &self,
        identity: TransferIdentity,
    ) -> Result<Option<SourceGeneration>> {
        let _update = self.representation_updates.lock().await;
        let binding = self.current_binding(&identity).await?;
        let key = identity.post().as_str().to_owned();
        let already_selected = self.selected().get(&key) == Some(&identity);
        let restored = self.generation_for(&key, identity.source().as_str()).await;
        self.selected().insert(key.clone(), identity);
        if !already_selected && restored.is_none() {
            self.replace_stored_generation(&key, &binding, None).await?;
        }
        Ok(restored)
    }

    pub async fn accept_generation(
        &self,
        identity: &TransferIdentity,
        generation: SourceGeneration,
    ) -> Result<()> {
        let _update = self.representation_updates.lock().await;
        let binding = self.current_binding(identity).await?;
        let key = identity.post().as_str();
        if self.selected().get(key) != Some(identity) {
            bail!("sparse transfer is no longer selected");
        }
        let current = self.generation_for(key, identity.source().as_str()).await;
        if current.as_ref() == Some(&generation) {
            return Ok(());
        }
        self.replace_stored_generation(key, &binding, Some(generation.clone()))
            .await?;
        self.source_generations.lock().await.insert(
            key.to_owned(),
            (identity.source().as_str().to_owned(), generation),
        );
        self.changed.notify_waiters();
        Ok(())
    }

    pub async fn write_range_for_generation_if_current(
        &self,
        identity: &TransferIdentity,
        generation: &SourceGeneration,
        offset: u64,
        bytes: &[u8],
    ) -> Result<bool> {
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

    pub async fn generation_is_current(
        &self,
        identity: &TransferIdentity,
        generation: &SourceGeneration,
    ) -> bool {
        self.transfer_is_current(identity).await
            && self
                .generation_for(identity.post().as_str(), identity.source().as_str())
                .await
                .as_ref()
                == Some(generation)
    }

    pub(super) async fn restore_generation(
        &self,
        binding: &ghostr_engine::representation::RepresentationBinding,
    ) -> Result<()> {
        let key = binding.post().as_str();
        let stored = disk::load(&self.paths.generation(key)).await.ok().flatten();
        let valid = stored.filter(|stored| {
            stored.representation == binding.representation().fingerprint()
                && binding.transfer(&stored.source).is_some()
        });
        if let Some(stored) = valid {
            self.source_generations
                .lock()
                .await
                .insert(key.to_owned(), (stored.source, stored.generation));
            return Ok(());
        }
        self.replace_stored_generation(key, binding, None).await
    }

    async fn current_binding(
        &self,
        identity: &TransferIdentity,
    ) -> Result<ghostr_engine::representation::RepresentationBinding> {
        let binding = self
            .representations
            .lock()
            .await
            .get(identity.post().as_str())
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("sparse representation is not bound"))?;
        if binding.transfer(identity.source().as_str()).as_ref() != Some(identity) {
            bail!("sparse transfer identity is stale");
        }
        Ok(binding)
    }

    async fn generation_for(&self, key: &str, source: &str) -> Option<SourceGeneration> {
        self.source_generations
            .lock()
            .await
            .get(key)
            .filter(|(known, _)| known == source)
            .map(|(_, generation)| generation.clone())
    }

    async fn replace_stored_generation(
        &self,
        key: &str,
        binding: &ghostr_engine::representation::RepresentationBinding,
        generation: Option<SourceGeneration>,
    ) -> Result<()> {
        self.source_generations.lock().await.remove(key);
        let mut entries = self.entries.lock().await;
        self.discard(&mut entries, key).await?;
        representation_disk::save(
            &self.paths.representation(key),
            binding.representation().fingerprint(),
        )
        .await?;
        if let Some(generation) = generation {
            let source = self
                .selected()
                .get(key)
                .map(|identity| identity.source().as_str().to_owned())
                .ok_or_else(|| anyhow::anyhow!("sparse transfer is not selected"))?;
            disk::save(
                &self.paths.generation(key),
                &StoredGeneration {
                    representation: binding.representation().fingerprint().to_owned(),
                    source,
                    generation,
                },
            )
            .await?;
        }
        Ok(())
    }
}
