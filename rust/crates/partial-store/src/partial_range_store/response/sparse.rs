use super::{PartialRangeStore, ResponseOpenResult, StoreAction};
use anyhow::Result;
use ghostr_engine::representation::{SourceGeneration, TransferIdentity};
use ghostr_engine::ByteRange;

impl PartialRangeStore {
    pub(super) async fn sparse_replacement_required(
        &self,
        identity: &TransferIdentity,
        generation: &SourceGeneration,
    ) -> Result<bool> {
        let key = identity.post().as_str();
        let mut entries = self.entries.lock().await;
        let covered = self
            .entry(&mut entries, key)
            .await?
            .manifest
            .covered_bytes();
        if covered == 0 {
            return Ok(false);
        }
        let current = self.source_generations.lock().await.get(key).cloned();
        Ok(current.as_ref() != Some(&(identity.source().as_str().to_owned(), generation.clone())))
    }

    pub(super) async fn open_sparse_alongside_single(
        &self,
        identity: &TransferIdentity,
        action: &StoreAction,
        generation: &SourceGeneration,
        range: ByteRange,
    ) -> Result<Option<ResponseOpenResult>> {
        let key = identity.post().as_str();
        if !self.single_response_is_active(key).await {
            return Ok(None);
        }
        let canonical = self.generation_for(key, identity.source().as_str()).await;
        if self.live_single_response_is_active(key).await || canonical.as_ref() != Some(generation)
        {
            return Ok(Some(ResponseOpenResult::RequiresIndependentObject));
        }
        self.selected().insert(key.to_owned(), identity.clone());
        self.register_sparse_response(identity, action, generation.clone(), range)
            .await
            .map(Some)
    }
}
