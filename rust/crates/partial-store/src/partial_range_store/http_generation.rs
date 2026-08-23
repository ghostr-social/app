use super::PartialRangeStore;
use crate::partial_range_completion::Completion;
use crate::partial_range_http_generation_disk::{self as disk, StoredHttpGeneration};
use crate::partial_range_representation_disk as representation_disk;
use anyhow::Result;
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::representation::{
    HttpGenerationAuthority, HttpGenerationKey, HttpGenerationLease, SourceGeneration,
    TransferIdentity,
};

mod recovery;

#[derive(Clone)]
pub(super) struct HttpGenerationState {
    source: String,
    key: Option<HttpGenerationKey>,
    authority: Option<HttpGenerationAuthority>,
}

impl PartialRangeStore {
    pub async fn apply_http_generation(
        &self,
        identity: &TransferIdentity,
        authority: HttpGenerationAuthority,
    ) -> Result<bool> {
        let key = identity.post().as_str();
        let _update = self.update_key(key).await?;
        let binding = self.current_binding(identity).await?;
        self.restore_http_generation(&binding).await?;
        if self.http_generation_matches(identity, &authority).await {
            self.adopt_runtime_authority(identity, authority).await;
            return Ok(true);
        }
        if self.http_generation_is_immutable(key).await? {
            return Ok(false);
        }
        self.revoke_single_response(key).await;
        let mut entries = self.entries.lock().await;
        self.discard_before_authority(&mut entries, key).await?;
        drop(entries);
        representation_disk::save(
            &self.paths.representation(key),
            binding.representation().fingerprint(),
        )
        .await?;
        self.persist_http_generation(identity, authority).await?;
        self.selected().insert(key.to_owned(), identity.clone());
        Ok(true)
    }

    pub(super) async fn http_generation_for(
        &self,
        identity: &TransferIdentity,
    ) -> Option<HttpGenerationLease> {
        let state = self.http_generations.lock().await;
        let known = state.get(identity.post().as_str())?;
        if known.source != identity.source().as_str() {
            return None;
        }
        match known.authority.as_ref()? {
            HttpGenerationAuthority::Trusted(lease) => Some(lease.clone()),
            HttpGenerationAuthority::Unknown(_) => None,
        }
    }

    pub(super) async fn http_generation_preserves_bytes(
        &self,
        identity: &TransferIdentity,
    ) -> bool {
        self.http_generations
            .lock()
            .await
            .get(identity.post().as_str())
            .is_some_and(|state| {
                state.source == identity.source().as_str()
                    && state
                        .key
                        .as_ref()
                        .and_then(HttpGenerationKey::validator)
                        .is_some()
            })
    }

    pub(super) async fn http_generation_matches_source(
        &self,
        identity: &TransferIdentity,
        generation: &SourceGeneration,
    ) -> bool {
        let validator = EvidenceValidator::strong_etag(generation.strong_etag());
        self.http_generations
            .lock()
            .await
            .get(identity.post().as_str())
            .is_some_and(|state| {
                state.source == identity.source().as_str()
                    && state.key.as_ref().is_some_and(|key| {
                        key.final_url() == generation.final_url()
                            && key.validator() == validator.as_ref()
                    })
            })
    }

    pub(super) async fn http_generation_is_current(
        &self,
        identity: &TransferIdentity,
        lease: &HttpGenerationLease,
    ) -> bool {
        self.http_generation_for(identity).await.as_ref() == Some(lease)
    }

    pub(super) async fn retire_http_generation(&self, key: &str) {
        self.http_generations.lock().await.remove(key);
        if let Err(error) =
            crate::partial_range_disk::remove_durable(&self.paths.http_generation(key)).await
        {
            log::warn!("Video store could not retire HTTP generation for {key}: {error:#}");
        }
    }

    async fn http_generation_matches(
        &self,
        identity: &TransferIdentity,
        authority: &HttpGenerationAuthority,
    ) -> bool {
        let states = self.http_generations.lock().await;
        let Some(state) = states.get(identity.post().as_str()) else {
            return false;
        };
        if state.source != identity.source().as_str() {
            return false;
        }
        match authority {
            HttpGenerationAuthority::Trusted(lease) => state.key.as_ref() == Some(lease.key()),
            HttpGenerationAuthority::Unknown(epoch) => {
                state.authority.as_ref() == Some(&HttpGenerationAuthority::Unknown(*epoch))
            }
        }
    }

    async fn adopt_runtime_authority(
        &self,
        identity: &TransferIdentity,
        authority: HttpGenerationAuthority,
    ) {
        if let Some(state) = self
            .http_generations
            .lock()
            .await
            .get_mut(identity.post().as_str())
        {
            state.authority = Some(authority);
        }
        self.selected()
            .insert(identity.post().as_str().to_owned(), identity.clone());
    }
    async fn http_generation_is_immutable(&self, key: &str) -> Result<bool> {
        let mut entries = self.entries.lock().await;
        Ok(self.entry(&mut entries, key).await?.completion == Some(Completion::Verified))
    }
    async fn persist_http_generation(
        &self,
        identity: &TransferIdentity,
        authority: HttpGenerationAuthority,
    ) -> Result<()> {
        let key = identity.post().as_str();
        let trusted = match &authority {
            HttpGenerationAuthority::Trusted(lease) => Some(lease.key().clone()),
            HttpGenerationAuthority::Unknown(_) => None,
        };
        if let Some(generation) = trusted.as_ref().filter(|key| key.validator().is_some()) {
            self.save_http_generation(identity, generation).await?;
        }
        self.http_generations.lock().await.insert(
            key.to_owned(),
            HttpGenerationState {
                source: identity.source().as_str().to_owned(),
                key: trusted,
                authority: Some(authority),
            },
        );
        Ok(())
    }
    async fn save_http_generation(
        &self,
        identity: &TransferIdentity,
        key: &HttpGenerationKey,
    ) -> Result<()> {
        let binding = self.current_binding(identity).await?;
        disk::save(
            &self.paths.http_generation(identity.post().as_str()),
            &StoredHttpGeneration {
                representation: binding.representation().fingerprint().to_owned(),
                source: identity.source().as_str().to_owned(),
                key: key.clone(),
            },
        )
        .await
    }
}
