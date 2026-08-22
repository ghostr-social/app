use super::SparseResponseState;
use crate::partial_range_store::sparse_intent::{self, SparseIntentAction};
use crate::partial_range_store::{PartialRangeStore, StoreAction};
use anyhow::{ensure, Result};
use ghostr_engine::representation::{SourceGeneration, TransferIdentity};
use sha2::Digest;

impl PartialRangeStore {
    pub(super) async fn sparse_state(
        &self,
        identity: &TransferIdentity,
        generation: &SourceGeneration,
        action: &StoreAction,
    ) -> Option<SparseResponseState> {
        self.sparse_response_actions
            .lock()
            .await
            .get(&action.id())
            .filter(|state| matches_state(state, identity, generation, action))
            .cloned()
    }

    pub(super) async fn install_sparse_intent(&self, state: &SparseResponseState) -> Result<()> {
        if state.intent_installed {
            return Ok(());
        }
        let binding = self.current_binding(&state.identity).await?;
        let stable_accounted = self.stable_accounted(&state.identity).await?;
        sparse_intent::add(
            &self.paths,
            state.identity.post().as_str(),
            SparseIntentAction {
                id: state.owner.id(),
                representation: binding.representation().fingerprint(),
                source: state.identity.source().as_str(),
                generation: &state.generation,
                range: state.range,
            },
            stable_accounted,
        )
        .await?;
        let mut responses = self.sparse_response_actions.lock().await;
        let known = exact_state_mut(&mut responses, &state.owner)?;
        known.intent_installed = true;
        Ok(())
    }

    async fn stable_accounted(&self, identity: &TransferIdentity) -> Result<u64> {
        let mut entries = self.entries.lock().await;
        Ok(self
            .entry(&mut entries, identity.post().as_str())
            .await?
            .accounted)
    }

    pub(super) async fn mark_sparse_attempt(&self, action: &StoreAction, bytes: u64) -> Result<()> {
        let mut responses = self.sparse_response_actions.lock().await;
        let state = exact_state_mut(&mut responses, action)?;
        state.pending = state.pending.saturating_add(bytes);
        state.dirty = true;
        Ok(())
    }

    pub(super) async fn record_sparse_write(
        &self,
        action: &StoreAction,
        bytes: &[u8],
    ) -> Result<()> {
        let mut responses = self.sparse_response_actions.lock().await;
        let state = exact_state_mut(&mut responses, action)?;
        state.hasher.update(bytes);
        state.received = state.received.saturating_add(bytes.len() as u64);
        state.next_offset = state.next_offset.saturating_add(bytes.len() as u64);
        state.dirty = false;
        Ok(())
    }

    pub(super) async fn mark_sparse_committed(
        &self,
        action: &StoreAction,
        bytes: u64,
    ) -> Result<()> {
        let mut responses = self.sparse_response_actions.lock().await;
        let state = exact_state_mut(&mut responses, action)?;
        ensure!(state.pending == bytes, "sparse pending accounting changed");
        state.pending = 0;
        state.committed = true;
        Ok(())
    }

    pub(super) async fn finish_sparse_intent(
        &self,
        key: &str,
        state: &SparseResponseState,
    ) -> Result<()> {
        let stable = self.stable_accounted(&state.identity).await?;
        self.finish_sparse_intent_with_total(key, state, stable)
            .await
    }

    pub(super) async fn finish_sparse_intent_with_total(
        &self,
        key: &str,
        state: &SparseResponseState,
        stable: u64,
    ) -> Result<()> {
        if !state.intent_installed {
            return Ok(());
        }
        let result = sparse_intent::commit(&self.paths, key, state.owner.id(), stable).await;
        if result.is_err() && !sparse_intent::exists(&self.paths, key).await? {
            return Ok(());
        }
        result
    }

    pub(super) async fn remove_sparse_state(&self, action: &StoreAction) {
        let mut responses = self.sparse_response_actions.lock().await;
        if responses
            .get(&action.id())
            .is_some_and(|state| state.owner.same_authority(action))
        {
            responses.remove(&action.id());
        }
    }

    pub(super) async fn sparse_action_is_current(
        &self,
        identity: &TransferIdentity,
        generation: &SourceGeneration,
        action: &StoreAction,
    ) -> bool {
        let registered = self
            .sparse_response_actions
            .lock()
            .await
            .get(&action.id())
            .is_some_and(|state| matches_state(state, identity, generation, action));
        registered && self.generation_is_current(identity, generation).await
    }

    pub(in crate::partial_range_store) async fn sparse_response_for_post(&self, key: &str) -> bool {
        self.sparse_response_actions
            .lock()
            .await
            .values()
            .any(|state| state.identity.post().as_str() == key)
    }
}

fn exact_state_mut<'a>(
    responses: &'a mut std::collections::HashMap<u64, SparseResponseState>,
    action: &StoreAction,
) -> Result<&'a mut SparseResponseState> {
    responses
        .get_mut(&action.id())
        .filter(|state| state.owner.same_authority(action))
        .ok_or_else(|| anyhow::anyhow!("sparse action disappeared"))
}

fn matches_state(
    state: &SparseResponseState,
    identity: &TransferIdentity,
    generation: &SourceGeneration,
    action: &StoreAction,
) -> bool {
    state.owner.same_authority(action)
        && state.identity == *identity
        && state.generation == *generation
}
