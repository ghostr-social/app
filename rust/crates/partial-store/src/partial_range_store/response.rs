use super::{PartialRangeStore, StoreAction};
use anyhow::Result;
use ghostr_engine::adaptive::WholeBodyContract;
use ghostr_engine::representation::{SourceGeneration, TransferIdentity};
use ghostr_engine::ByteRange;
use sha2::{Digest, Sha256};

mod sparse;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponseOpenResult {
    Opened,
    RequiresIndependentObject,
    Stale,
}

impl PartialRangeStore {
    pub async fn open_sparse_response(
        &self,
        identity: &TransferIdentity,
        action: &StoreAction,
        generation: SourceGeneration,
        range: ByteRange,
    ) -> Result<ResponseOpenResult> {
        if !action.is_active() || action.identity() != identity {
            return Ok(ResponseOpenResult::Stale);
        }
        let _update = self.update_key(identity.post().as_str()).await?;
        if !action.is_active() {
            return Ok(ResponseOpenResult::Stale);
        }
        self.current_binding(identity).await?;
        if self
            .sparse_response_is_open(identity, action, &generation, range)
            .await
        {
            return Ok(ResponseOpenResult::Opened);
        }
        if let Some(opened) = self
            .open_sparse_alongside_single(identity, action, &generation, range)
            .await?
        {
            return Ok(opened);
        }
        if self
            .sparse_replacement_required(identity, &generation)
            .await?
        {
            return Ok(ResponseOpenResult::RequiresIndependentObject);
        }
        self.accept_generation_locked(identity, generation.clone())
            .await?;
        self.selected()
            .insert(identity.post().as_str().to_owned(), identity.clone());
        self.register_sparse_response(identity, action, generation, range)
            .await
    }

    pub async fn open_single_response_for_action(
        &self,
        identity: &TransferIdentity,
        action: &StoreAction,
        contract: WholeBodyContract,
    ) -> Result<ResponseOpenResult> {
        if !action.is_active() || action.identity() != identity {
            return Ok(ResponseOpenResult::Stale);
        }
        let _update = self.update_key(identity.post().as_str()).await?;
        if !action.is_active() {
            return Ok(ResponseOpenResult::Stale);
        }
        self.current_binding(identity).await?;
        self.open_single_response_action_locked(
            identity,
            super::single_response::ResponseOwner::Granted(action.clone()),
            contract,
        )
        .await
    }

    pub(super) async fn abort_response_for_action(&self, action: &StoreAction) -> Result<()> {
        let sparse = self
            .sparse_response_actions
            .lock()
            .await
            .get(&action.id())
            .filter(|state| state.owner.same_authority(action))
            .cloned();
        if let Some(state) = sparse {
            self.finish_sparse_response(&state.identity, &state.generation, action)
                .await?;
        }
        self.finish_single_response_for_action(action.identity(), action, None, false)
            .await?;
        Ok(())
    }

    async fn sparse_response_is_open(
        &self,
        identity: &TransferIdentity,
        action: &StoreAction,
        generation: &SourceGeneration,
        range: ByteRange,
    ) -> bool {
        self.sparse_response_actions
            .lock()
            .await
            .get(&action.id())
            .is_some_and(|state| {
                state.owner.same_authority(action)
                    && state.identity == *identity
                    && state.generation == *generation
                    && state.range == range
            })
    }

    async fn register_sparse_response(
        &self,
        identity: &TransferIdentity,
        action: &StoreAction,
        generation: SourceGeneration,
        range: ByteRange,
    ) -> Result<ResponseOpenResult> {
        if range.is_empty()
            || range.len() > ghostr_engine::adaptive::REQUEST_SLICE_BYTES
            || range.end > generation.total_bytes()
        {
            return Ok(ResponseOpenResult::Stale);
        }
        let mut entries = self.entries.lock().await;
        let missing = self
            .entry(&mut entries, identity.post().as_str())
            .await?
            .manifest
            .missing_within(&(range.start..range.end));
        drop(entries);
        let exact_hole =
            missing.len() == 1 && missing[0].start == range.start && missing[0].end == range.end;
        if !exact_hole {
            return Ok(ResponseOpenResult::Stale);
        }
        let mut responses = self.sparse_response_actions.lock().await;
        if responses.values().any(|known| {
            known.owner.is_active()
                && known.identity.post() == identity.post()
                && known.range.start < range.end
                && range.start < known.range.end
        }) {
            return Ok(ResponseOpenResult::Stale);
        }
        responses.insert(
            action.id(),
            super::generation::SparseResponseState {
                owner: action.clone(),
                identity: identity.clone(),
                generation,
                range,
                next_offset: range.start,
                received: 0,
                pending: 0,
                hasher: Sha256::new(),
                intent_installed: false,
                dirty: false,
                committed: false,
            },
        );
        Ok(ResponseOpenResult::Opened)
    }
}
