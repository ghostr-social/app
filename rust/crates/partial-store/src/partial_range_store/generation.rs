use super::PartialRangeStore;
use anyhow::{bail, Result};
use ghostr_engine::representation::{SourceGeneration, TransferIdentity};

mod extent;
mod recovery;
mod sparse_response;
#[cfg(any(test, feature = "test"))]
mod test_support;
pub(super) use sparse_response::SparseResponseState;

impl PartialRangeStore {
    /// # Errors
    ///
    /// Returns an error when the transfer binding is stale or its state cannot be observed.
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
        let incoming = (identity.source().as_str(), generation.clone());
        if current.is_none()
            && self
                .http_generation_matches_source(identity, &generation)
                .await
        {
            self.persist_generation(key, &binding, Some(incoming))
                .await?;
        } else {
            self.replace_stored_generation(key, &binding, Some(incoming))
                .await?;
        }
        self.selected().insert(key.to_owned(), identity.to_owned());
        self.source_generations.lock().await.insert(
            key.to_owned(),
            (identity.source().as_str().to_owned(), generation),
        );
        self.changed.notify_waiters();
        Ok(())
    }

    async fn generation_is_current(
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
