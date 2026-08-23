use super::PartialRangeStore;
use crate::partial_range_disk as range_disk;
use crate::partial_range_generation_disk::{self as disk, StoredGeneration};
use crate::partial_range_representation_disk as representation_disk;
use anyhow::{bail, Result};
use ghostr_engine::representation::{RepresentationBinding, SourceGeneration, TransferIdentity};

impl PartialRangeStore {
    pub(in crate::partial_range_store) async fn restore_compatible_generation(
        &self,
        binding: &RepresentationBinding,
    ) -> Result<()> {
        let key = binding.post().as_str();
        if self.source_generations.lock().await.contains_key(key) {
            return Ok(());
        }
        let Some(stored) = disk::load(&self.paths.generation(key)).await? else {
            return Ok(());
        };
        if !generation_matches_binding(&stored, binding) {
            return Ok(());
        }
        let entries = self.entries.lock().await;
        let Some(entry) = entries.get(key) else {
            return Ok(());
        };
        if entry.completion.is_some()
            || entry.manifest.total_len() != Some(stored.generation.total_bytes())
        {
            return Ok(());
        }
        self.source_generations
            .lock()
            .await
            .insert(key.to_owned(), (stored.source, stored.generation));
        Ok(())
    }

    pub(in crate::partial_range_store) async fn retire_generation(&self, key: &str) {
        self.source_generations.lock().await.remove(key);
        self.selected().remove(key);
        if let Err(error) = range_disk::remove_durable(&self.paths.generation(key)).await {
            log::warn!("Video store could not retire generation for {key}: {error:#}");
        }
    }

    pub(in crate::partial_range_store) async fn current_binding(
        &self,
        identity: &TransferIdentity,
    ) -> Result<RepresentationBinding> {
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

    pub(in crate::partial_range_store) async fn generation_for(
        &self,
        key: &str,
        source: &str,
    ) -> Option<SourceGeneration> {
        self.source_generations
            .lock()
            .await
            .get(key)
            .filter(|(known, _)| known == source)
            .map(|(_, generation)| generation.clone())
    }

    pub(in crate::partial_range_store) async fn replace_stored_generation(
        &self,
        key: &str,
        binding: &RepresentationBinding,
        generation: Option<(&str, SourceGeneration)>,
    ) -> Result<()> {
        self.source_generations.lock().await.remove(key);
        let mut entries = self.entries.lock().await;
        if self.entry(&mut entries, key).await?.completion.is_some() {
            bail!("cannot replace a finalized video");
        }
        self.discard_before_authority(&mut entries, key).await?;
        drop(entries);
        representation_disk::save(
            &self.paths.representation(key),
            binding.representation().fingerprint(),
        )
        .await?;
        self.persist_generation(key, binding, generation).await
    }

    pub(in crate::partial_range_store) async fn persist_generation(
        &self,
        key: &str,
        binding: &RepresentationBinding,
        generation: Option<(&str, SourceGeneration)>,
    ) -> Result<()> {
        let Some((source, generation)) = generation else {
            return Ok(());
        };
        let total = generation.total_bytes();
        disk::save(
            &self.paths.generation(key),
            &StoredGeneration {
                representation: binding.representation().fingerprint().to_owned(),
                source: source.to_owned(),
                generation,
            },
        )
        .await?;
        if !self.install_generation_total(key, total).await? {
            bail!("sparse generation conflicts with its stored extent");
        }
        Ok(())
    }

    pub(in crate::partial_range_store) async fn ensure_sparse_mutable(
        &self,
        key: &str,
    ) -> Result<()> {
        let mut entries = self.entries.lock().await;
        if self.entry(&mut entries, key).await?.completion.is_some() {
            bail!("cannot replace a finalized video");
        }
        Ok(())
    }
}

fn generation_matches_binding(stored: &StoredGeneration, binding: &RepresentationBinding) -> bool {
    stored.representation == binding.representation().fingerprint()
        && binding.transfer(&stored.source).is_some()
}
