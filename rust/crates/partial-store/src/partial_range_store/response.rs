use super::{PartialRangeStore, StoreAction};
use anyhow::Result;
use ghostr_engine::representation::{SourceGeneration, TransferIdentity};
use ghostr_engine::ByteRange;
use sha2::{Digest as _, Sha256};

mod sparse;
mod whole;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResponseOpenResult {
    Opened,
    RequiresIndependentObject,
    Stale,
}

impl PartialRangeStore {
    /// # Errors
    ///
    /// Returns an error when response authority, range geometry, or persistence validation fails.
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
        if !valid_sparse_range(&generation, range) {
            return Ok(ResponseOpenResult::Stale);
        }
        if !self.sparse_range_is_missing(identity, range).await? {
            return Ok(ResponseOpenResult::Stale);
        }
        let mut responses = self.sparse_response_actions.lock().await;
        if has_active_overlap(responses.values(), identity, range) {
            return Ok(ResponseOpenResult::Stale);
        }
        responses.insert(
            action.id(),
            sparse_response_state(identity, action, generation, range),
        );
        Ok(ResponseOpenResult::Opened)
    }

    async fn sparse_range_is_missing(
        &self,
        identity: &TransferIdentity,
        range: ByteRange,
    ) -> Result<bool> {
        let mut entries = self.entries.lock().await;
        let missing = self
            .entry(&mut entries, identity.post().as_str())
            .await?
            .manifest
            .missing_within(&(range.start..range.end));
        Ok(matches!(
            missing.as_slice(),
            [span] if span.start == range.start && span.end == range.end
        ))
    }
}

fn valid_sparse_range(generation: &SourceGeneration, range: ByteRange) -> bool {
    !range.is_empty()
        && range.len() <= ghostr_engine::adaptive::REQUEST_SLICE_BYTES
        && range.end <= generation.total_bytes()
}

fn has_active_overlap<'a>(
    responses: impl Iterator<Item = &'a super::generation::SparseResponseState>,
    identity: &TransferIdentity,
    range: ByteRange,
) -> bool {
    responses.into_iter().any(|known| {
        known.owner.is_active()
            && known.identity.post() == identity.post()
            && known.range.start < range.end
            && range.start < known.range.end
    })
}

fn sparse_response_state(
    identity: &TransferIdentity,
    action: &StoreAction,
    generation: SourceGeneration,
    range: ByteRange,
) -> super::generation::SparseResponseState {
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
    }
}
