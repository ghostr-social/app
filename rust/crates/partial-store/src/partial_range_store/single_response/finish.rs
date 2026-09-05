use super::{
    accepted_total, PartialRangeStore, ResponseOwnerRef, SingleResponseState, SingleResponseStorage,
};
use crate::partial_range_store::StoreAction;
use anyhow::{Context as _, Result};
use ghostr_engine::representation::{RepresentationBinding, TransferIdentity};

mod live;
mod resume;
#[cfg(any(test, feature = "test"))]
mod test_support;

impl PartialRangeStore {
    /// # Errors
    ///
    /// Returns an error when the action is stale or response completion cannot be committed.
    pub async fn finish_single_response_for_action(
        &self,
        identity: &TransferIdentity,
        action: &StoreAction,
        total: Option<u64>,
        complete: bool,
    ) -> Result<bool> {
        self.finish_single_response_owned(
            identity,
            ResponseOwnerRef::Granted(action),
            total,
            complete,
        )
        .await
    }

    async fn finish_single_response_owned(
        &self,
        identity: &TransferIdentity,
        owner: ResponseOwnerRef<'_>,
        total: Option<u64>,
        complete: bool,
    ) -> Result<bool> {
        let _update = self.update_key(identity.post().as_str()).await?;
        let Some(state) = self.single_response_for_finish(identity, owner).await else {
            return Ok(false);
        };
        let binding = self.current_binding(identity).await?;
        let known_owner = state.owner.clone();
        let result = self
            .finish_single_response_state(&binding, state, total, complete)
            .await;
        if result.is_ok() {
            self.remove_single_response_owner(identity, known_owner.as_ref())
                .await;
        }
        result
    }

    async fn finish_single_response_state(
        &self,
        binding: &RepresentationBinding,
        state: SingleResponseState,
        total: Option<u64>,
        complete: bool,
    ) -> Result<bool> {
        let Some(total) = accepted_total(state.contract, total, complete) else {
            return self.abort_single_response(binding, state).await;
        };
        if !state.owner.claim_publication() {
            return self.abort_single_response(binding, state).await;
        }
        let identity = state.identity.clone();
        let result = match state.storage {
            SingleResponseStorage::Memory => {
                self.finish_transient(binding.post().as_str(), total).await
            }
            SingleResponseStorage::Live { started } => {
                self.finish_live_single_response(binding, total, started, true)
                    .await
            }
            SingleResponseStorage::Staged { received } => {
                self.finish_staged_single_response(binding, total, &state, received, true)
                    .await
            }
        };
        if matches!(result, Ok(true)) {
            self.selected()
                .insert(binding.post().as_str().to_owned(), identity);
        }
        result
    }

    async fn abort_single_response(
        &self,
        binding: &RepresentationBinding,
        state: SingleResponseState,
    ) -> Result<bool> {
        match state.storage {
            SingleResponseStorage::Memory => {
                self.abort_transient(binding.post().as_str()).await?;
            }
            SingleResponseStorage::Live { .. } => {
                if self.retain_versioned_prefix(binding, &state).await? {
                    return Ok(true);
                }
                self.rollback_live_single_response(binding).await?;
            }
            SingleResponseStorage::Staged { .. } => {
                self.remove_staged_single_response(&state).await?;
            }
        }
        Ok(false)
    }

    async fn finish_staged_single_response(
        &self,
        binding: &RepresentationBinding,
        total: u64,
        state: &SingleResponseState,
        received: u64,
        exact: bool,
    ) -> Result<bool> {
        if exact && received == total {
            if matches!(
                &state.authority,
                super::SingleResponseAuthority::ActionScoped
            ) {
                self.commit_session_response(binding, state, total).await?;
                return Ok(true);
            }
            let policy = self.staged_commit_policy(state, total).await?;
            self.commit_staged_single_response(binding, total, policy)
                .await
                .context("commit complete staged response")?;
            return Ok(true);
        }
        self.remove_staged_single_response(state).await?;
        Ok(false)
    }

    async fn remove_single_response_owner(
        &self,
        identity: &TransferIdentity,
        owner: ResponseOwnerRef<'_>,
    ) {
        let mut actions = self.single_response_actions.lock().await;
        if actions
            .get(identity.post().as_str())
            .is_some_and(|state| state.owner.matches(owner))
        {
            actions.remove(identity.post().as_str());
        }
    }
}
