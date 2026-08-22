use super::{
    save_binding, PartialRangeStore, ResponseOwnerRef, SingleResponseState, SingleResponseStorage,
};
use crate::partial_range_disk as disk;
use crate::partial_range_store::StoreAction;
use anyhow::{bail, Context, Result};
use ghostr_engine::adaptive::WholeBodyContract;
use ghostr_engine::representation::{RepresentationBinding, TransferIdentity};

impl PartialRangeStore {
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
            complete && action.is_active(),
        )
        .await
    }

    pub async fn finish_single_response(
        &self,
        identity: &TransferIdentity,
        action: u64,
        total: Option<u64>,
        complete: bool,
    ) -> Result<bool> {
        self.finish_single_response_owned(
            identity,
            ResponseOwnerRef::Legacy(action),
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
        let Some(state) = self.current_single_response(identity, owner).await else {
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
        let identity = state.identity.clone();
        let result = match state.storage {
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
            SingleResponseStorage::Live { .. } => {
                self.rollback_live_single_response(binding).await?;
            }
            SingleResponseStorage::Staged { .. } => {
                self.remove_staged_single_response(&state).await?;
            }
        }
        Ok(false)
    }

    async fn finish_live_single_response(
        &self,
        binding: &RepresentationBinding,
        total: u64,
        started: bool,
        exact: bool,
    ) -> Result<bool> {
        if exact && started {
            return match self.seal_live_single_response(binding, total).await {
                Ok(sealed) => Ok(sealed),
                Err(error) => {
                    self.rollback_live_single_response(binding).await?;
                    Err(error)
                }
            };
        }
        self.rollback_live_single_response(binding).await?;
        Ok(false)
    }

    async fn rollback_live_single_response(&self, binding: &RepresentationBinding) -> Result<()> {
        let mut entries = self.entries.lock().await;
        self.discard_before_authority(&mut entries, binding.post().as_str())
            .await?;
        save_binding(self, binding).await?;
        self.changed.notify_waiters();
        Ok(())
    }

    async fn seal_live_single_response(
        &self,
        binding: &RepresentationBinding,
        total: u64,
    ) -> Result<bool> {
        let key = binding.post().as_str();
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        entry.manifest.set_total_len(total)?;
        if !entry.manifest.is_complete() {
            bail!("single response ended without the complete object");
        }
        disk::save_manifest(&self.paths.manifest(key), &entry.manifest).await?;
        self.changed.notify_waiters();
        Ok(true)
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
            if let Err(error) = self.commit_staged_single_response(binding, total).await {
                self.remove_staged_single_response(state)
                    .await
                    .with_context(|| format!("clean up failed staged commit: {error:#}"))?;
                self.remove_single_response_owner(&state.identity, state.owner.as_ref())
                    .await;
                return Err(error);
            }
            return Ok(true);
        }
        self.remove_staged_single_response(state).await?;
        Ok(false)
    }

    pub(super) async fn remove_single_response_owner(
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

fn accepted_total(contract: WholeBodyContract, total: Option<u64>, complete: bool) -> Option<u64> {
    let total = total.filter(|total| complete && *total > 0)?;
    match contract {
        WholeBodyContract::Exact { expected_bytes } if total == expected_bytes => Some(total),
        WholeBodyContract::Capped { maximum_bytes } if total <= maximum_bytes => Some(total),
        _ => None,
    }
}
