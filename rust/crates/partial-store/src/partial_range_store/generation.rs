use super::PartialRangeStore;
use anyhow::{bail, Result};
use ghostr_engine::representation::{SourceGeneration, TransferIdentity};

mod extent;
mod recovery;
mod sparse_response;
pub(super) use sparse_response::SparseResponseState;

impl PartialRangeStore {
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
        if restored.is_none() {
            self.replace_stored_generation(&key, &binding, None).await?;
        }
        self.selected().insert(key, identity);
        Ok(restored)
    }

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

    pub async fn continuation_for(
        &self,
        identity: &TransferIdentity,
    ) -> Result<Option<SourceGeneration>> {
        let _update = self.observe_key(identity.post().as_str()).await?;
        self.current_binding(identity).await?;
        Ok(self
            .generation_for(identity.post().as_str(), identity.source().as_str())
            .await)
    }

    pub(super) async fn accept_generation_locked(
        &self,
        identity: &TransferIdentity,
        generation: SourceGeneration,
    ) -> Result<()> {
        let binding = self.current_binding(identity).await?;
        let key = identity.post().as_str();
        let live_response_started = self.live_single_response_started(key).await;
        self.cancel_single_response(key).await?;
        if live_response_started {
            let mut entries = self.entries.lock().await;
            self.discard(&mut entries, key).await?;
        }
        let current = self.generation_for(key, identity.source().as_str()).await;
        if current.as_ref() == Some(&generation) {
            if !self
                .install_generation_total(key, generation.total_bytes())
                .await?
            {
                bail!("sparse generation conflicts with its stored extent");
            }
            return Ok(());
        }
        self.replace_stored_generation(
            key,
            &binding,
            Some((identity.source().as_str(), generation.clone())),
        )
        .await?;
        self.selected().insert(key.to_owned(), identity.to_owned());
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

    pub async fn generation_is_current(
        &self,
        identity: &TransferIdentity,
        generation: &SourceGeneration,
    ) -> bool {
        self.transfer_is_current(identity).await
            && !self
                .live_single_response_is_active(identity.post().as_str())
                .await
            && self
                .generation_for(identity.post().as_str(), identity.source().as_str())
                .await
                .as_ref()
                == Some(generation)
    }
}
