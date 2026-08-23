use super::HttpGenerationState;
use crate::partial_range_http_generation_disk::{
    self as disk, StoredHttpGeneration, StoredHttpGenerationLoad,
};
use crate::partial_range_representation_disk as representation_disk;
use crate::partial_range_store::PartialRangeStore;
use anyhow::Result;
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::representation::{HttpGenerationKey, RepresentationBinding, SourceGeneration};

impl PartialRangeStore {
    pub(in crate::partial_range_store) async fn restore_http_generation(
        &self,
        binding: &RepresentationBinding,
    ) -> Result<()> {
        let key = binding.post().as_str();
        if self.http_generations.lock().await.contains_key(key) {
            return Ok(());
        }
        if self.http_generation_is_immutable(key).await? {
            return Ok(());
        }
        self.restore_compatible_generation(binding).await?;
        match disk::load(&self.paths.http_generation(key)).await? {
            StoredHttpGenerationLoad::Missing => self.migrate_legacy_http(binding).await,
            StoredHttpGenerationLoad::Valid(stored) => {
                self.restore_stored_http(binding, stored).await
            }
            StoredHttpGenerationLoad::Invalid => self.discard_untrusted_http_bytes(binding).await,
        }
    }

    async fn migrate_legacy_http(&self, binding: &RepresentationBinding) -> Result<()> {
        let key = binding.post().as_str();
        let Some(stored) = self.legacy_http_generation(binding).await else {
            return self.discard_untrusted_http_bytes(binding).await;
        };
        disk::save(&self.paths.http_generation(key), &stored).await?;
        self.adopt_stored_http_generation(key, stored).await
    }

    async fn legacy_http_generation(
        &self,
        binding: &RepresentationBinding,
    ) -> Option<StoredHttpGeneration> {
        let key = binding.post().as_str();
        let (source, generation) = self.source_generations.lock().await.get(key).cloned()?;
        Some(StoredHttpGeneration {
            representation: binding.representation().fingerprint().to_owned(),
            source,
            key: legacy_key(&generation)?,
        })
    }

    async fn adopt_stored_http_generation(
        &self,
        key: &str,
        stored: StoredHttpGeneration,
    ) -> Result<()> {
        self.http_generations.lock().await.insert(
            key.to_owned(),
            HttpGenerationState {
                source: stored.source,
                key: Some(stored.key),
                authority: None,
            },
        );
        Ok(())
    }

    async fn restore_stored_http(
        &self,
        binding: &RepresentationBinding,
        stored: StoredHttpGeneration,
    ) -> Result<()> {
        let valid = stored.key.validator().is_some()
            && stored.representation == binding.representation().fingerprint()
            && binding.transfer(&stored.source).is_some()
            && self.legacy_agrees(binding, &stored).await?;
        if !valid {
            return self.discard_untrusted_http_bytes(binding).await;
        }
        self.adopt_stored_http_generation(binding.post().as_str(), stored)
            .await
    }

    async fn legacy_agrees(
        &self,
        binding: &RepresentationBinding,
        stored: &StoredHttpGeneration,
    ) -> Result<bool> {
        let path = self.paths.generation(binding.post().as_str());
        let present = tokio::fs::try_exists(path).await?;
        let legacy = self
            .source_generations
            .lock()
            .await
            .get(binding.post().as_str())
            .cloned();
        Ok(match (present, legacy) {
            (false, None) => true,
            (true, Some((source, generation))) => legacy_matches(stored, &source, &generation),
            _ => false,
        })
    }

    async fn discard_untrusted_http_bytes(&self, binding: &RepresentationBinding) -> Result<()> {
        let key = binding.post().as_str();
        self.revoke_single_response(key).await;
        let mut entries = self.entries.lock().await;
        self.discard_before_authority(&mut entries, key).await?;
        drop(entries);
        representation_disk::save(
            &self.paths.representation(key),
            binding.representation().fingerprint(),
        )
        .await
    }
}

fn legacy_key(generation: &SourceGeneration) -> Option<HttpGenerationKey> {
    let validator = EvidenceValidator::strong_etag(generation.strong_etag())?;
    HttpGenerationKey::try_new(generation.final_url(), Some(validator)).ok()
}

fn legacy_matches(
    stored: &StoredHttpGeneration,
    source: &str,
    generation: &SourceGeneration,
) -> bool {
    stored.source == source
        && stored.key.final_url() == generation.final_url()
        && stored.key.validator()
            == EvidenceValidator::strong_etag(generation.strong_etag()).as_ref()
}
