use super::PartialRangeStore;
use crate::partial_range_disk as disk;
use crate::partial_range_manifest::RangeManifest;
use crate::partial_range_representation_disk as representation_disk;
use crate::partial_range_store::response::ResponseOpenResult;
use crate::partial_range_store::StoreAction;
use anyhow::{ensure, Result};
use ghostr_engine::adaptive::WholeBodyContract;
use ghostr_engine::representation::{RepresentationBinding, TransferIdentity};
use std::collections::HashMap;

mod finish;
mod staged;

#[derive(Clone)]
pub(super) struct SingleResponseState {
    pub(super) owner: ResponseOwner,
    pub(super) identity: TransferIdentity,
    pub(super) contract: WholeBodyContract,
    pub(super) storage: SingleResponseStorage,
}

#[derive(Clone)]
pub(super) enum ResponseOwner {
    Legacy(u64),
    Granted(StoreAction),
}

#[derive(Clone, Copy)]
pub(super) enum ResponseOwnerRef<'a> {
    Legacy(u64),
    Granted(&'a StoreAction),
}

impl ResponseOwner {
    pub(super) fn is_active(&self) -> bool {
        match self {
            Self::Legacy(_) => true,
            Self::Granted(action) => action.is_active(),
        }
    }

    pub(super) fn matches(&self, owner: ResponseOwnerRef<'_>) -> bool {
        match (self, owner) {
            (Self::Legacy(known), ResponseOwnerRef::Legacy(seen)) => *known == seen,
            (Self::Granted(known), ResponseOwnerRef::Granted(seen)) => known.same_authority(seen),
            _ => false,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum SingleResponseStorage {
    Live { started: bool },
    Staged { received: u64 },
}

impl PartialRangeStore {
    pub async fn begin_single_response_for_action(
        &self,
        identity: &TransferIdentity,
        action: &StoreAction,
        contract: WholeBodyContract,
    ) -> Result<bool> {
        Ok(matches!(
            self.open_single_response_for_action(identity, action, contract)
                .await?,
            ResponseOpenResult::Opened
        ))
    }

    pub async fn begin_single_response(
        &self,
        identity: &TransferIdentity,
        action: u64,
        contract: WholeBodyContract,
    ) -> Result<bool> {
        ensure!(
            contract.maximum_bytes() > 0,
            "single response limit must be positive"
        );
        let _update = self.update_key(identity.post().as_str()).await?;
        self.current_binding(identity).await?;
        if self.selected().get(identity.post().as_str()) != Some(identity) {
            return Ok(false);
        }
        Ok(matches!(
            self.open_single_response_action_locked(
                identity,
                ResponseOwner::Legacy(action),
                contract,
            )
            .await?,
            ResponseOpenResult::Opened
        ))
    }

    pub(super) async fn open_single_response_action_locked(
        &self,
        identity: &TransferIdentity,
        owner: ResponseOwner,
        contract: WholeBodyContract,
    ) -> Result<ResponseOpenResult> {
        ensure!(
            contract.maximum_bytes() > 0,
            "single response limit must be positive"
        );
        let binding = self.current_binding(identity).await?;
        let key = identity.post().as_str();
        self.retry_inactive_single_response_locked(key).await?;
        if let Some(known) = self.single_response_actions.lock().await.get(key) {
            let same = known.owner.matches(owner.as_ref())
                && known.identity == *identity
                && known.contract == contract;
            return Ok(if same {
                ResponseOpenResult::Opened
            } else {
                ResponseOpenResult::RequiresIndependentObject
            });
        }
        let storage = self.single_response_storage(key, contract).await?;
        if matches!(storage, SingleResponseStorage::Live { .. })
            && self.sparse_response_for_post(key).await
        {
            return Ok(ResponseOpenResult::RequiresIndependentObject);
        }
        if let WholeBodyContract::Exact { expected_bytes } = contract {
            if matches!(storage, SingleResponseStorage::Live { .. }) {
                self.selected().insert(key.to_owned(), identity.clone());
                self.install_provisional_total(&binding, expected_bytes)
                    .await?;
            }
        }
        self.single_response_actions.lock().await.insert(
            key.to_owned(),
            SingleResponseState {
                owner,
                identity: identity.clone(),
                contract,
                storage,
            },
        );
        Ok(ResponseOpenResult::Opened)
    }

    pub async fn write_single_response_if_current(
        &self,
        identity: &TransferIdentity,
        action: u64,
        offset: u64,
        bytes: &[u8],
    ) -> Result<bool> {
        self.write_single_response(
            identity,
            ResponseOwnerRef::Legacy(action),
            None,
            offset,
            bytes,
        )
        .await
    }

    pub async fn write_single_response_for_action(
        &self,
        identity: &TransferIdentity,
        action: &StoreAction,
        offset: u64,
        bytes: &[u8],
    ) -> Result<bool> {
        if !action.is_active() {
            return Ok(false);
        }
        self.write_single_response(
            identity,
            ResponseOwnerRef::Granted(action),
            Some(action),
            offset,
            bytes,
        )
        .await
    }

    async fn write_single_response(
        &self,
        identity: &TransferIdentity,
        owner: ResponseOwnerRef<'_>,
        reservation: Option<&StoreAction>,
        offset: u64,
        bytes: &[u8],
    ) -> Result<bool> {
        let _update = self.update_key(identity.post().as_str()).await?;
        let Some(state) = self.current_single_response(identity, owner).await else {
            return Ok(false);
        };
        match state.storage {
            SingleResponseStorage::Live { .. } => {
                self.write_live_single_response(identity, state, reservation, offset, bytes)
                    .await
            }
            SingleResponseStorage::Staged { .. } => {
                self.write_staged_single_response(identity, state, reservation, offset, bytes)
                    .await
            }
        }
    }

    async fn write_live_single_response(
        &self,
        identity: &TransferIdentity,
        state: SingleResponseState,
        reservation: Option<&StoreAction>,
        offset: u64,
        bytes: &[u8],
    ) -> Result<bool> {
        ensure!(!bytes.is_empty(), "single response write is empty");
        ensure!(
            offset.saturating_add(bytes.len() as u64) <= state.contract.maximum_bytes(),
            "response exceeds length"
        );
        let mut entries = self.entries.lock().await;
        match reservation {
            Some(action) => {
                self.write_range_locked_for_action(
                    &mut entries,
                    identity.post().as_str(),
                    action,
                    offset,
                    bytes,
                )
                .await?
            }
            None => {
                self.write_range_locked(&mut entries, identity.post().as_str(), offset, bytes)
                    .await?
            }
        }
        self.mark_live_started(identity.post().as_str(), state.owner.as_ref())
            .await;
        Ok(true)
    }

    async fn install_provisional_total(
        &self,
        binding: &RepresentationBinding,
        total: u64,
    ) -> Result<()> {
        let key = binding.post().as_str();
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        if entry.manifest.total_len() == Some(total) && entry.manifest.covered_bytes() == 0 {
            return Ok(());
        }
        let mut manifest = RangeManifest::default();
        manifest.set_total_len(total)?;
        disk::save_manifest(&self.paths.manifest(key), &manifest).await?;
        entry.manifest = manifest;
        self.advance_content_revision(key).await;
        self.changed.notify_waiters();
        Ok(())
    }

    async fn single_response_storage(
        &self,
        key: &str,
        contract: WholeBodyContract,
    ) -> Result<SingleResponseStorage> {
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        ensure!(
            entry.completion.is_none(),
            "cannot replace a finalized video"
        );
        let has_sparse_generation = self.source_generations.lock().await.contains_key(key);
        let can_stream_live = entry.accounted == 0
            && !has_sparse_generation
            && matches!(contract, WholeBodyContract::Exact { .. });
        Ok(match can_stream_live {
            true => SingleResponseStorage::Live { started: false },
            false => SingleResponseStorage::Staged { received: 0 },
        })
    }

    async fn current_single_response(
        &self,
        identity: &TransferIdentity,
        owner: ResponseOwnerRef<'_>,
    ) -> Option<SingleResponseState> {
        let state = self
            .single_response_actions
            .lock()
            .await
            .get(identity.post().as_str())
            .cloned()
            .filter(|state| state.owner.matches(owner) && state.identity == *identity)?;
        self.current_binding(identity).await.ok().map(|_| state)
    }

    async fn mark_live_started(&self, key: &str, owner: ResponseOwnerRef<'_>) {
        let mut actions = self.single_response_actions.lock().await;
        let Some(state) = actions
            .get_mut(key)
            .filter(|state| state.owner.matches(owner))
        else {
            return;
        };
        state.storage = SingleResponseStorage::Live { started: true };
    }

    pub(super) async fn cancel_single_response(&self, key: &str) -> Result<()> {
        let state = self.single_response_actions.lock().await.get(key).cloned();
        if let Some(state) = state.as_ref() {
            if matches!(state.storage, SingleResponseStorage::Staged { .. }) {
                self.remove_staged_single_response(state).await?;
            }
        }
        if let Some(state) = state {
            let mut actions = self.single_response_actions.lock().await;
            if actions
                .get(key)
                .is_some_and(|known| known.owner.matches(state.owner.as_ref()))
            {
                actions.remove(key);
            }
        }
        Ok(())
    }

    pub(super) async fn live_single_response_started(&self, key: &str) -> bool {
        self.single_response_actions
            .lock()
            .await
            .get(key)
            .is_some_and(|state| {
                state.owner.is_active()
                    && matches!(state.storage, SingleResponseStorage::Live { started: true })
            })
    }

    pub(super) async fn single_response_is_active(&self, key: &str) -> bool {
        self.single_response_actions
            .lock()
            .await
            .get(key)
            .is_some_and(|state| state.owner.is_active())
    }

    pub(super) async fn live_single_response_is_active(&self, key: &str) -> bool {
        self.single_response_actions
            .lock()
            .await
            .get(key)
            .is_some_and(|state| {
                state.owner.is_active()
                    && matches!(state.storage, SingleResponseStorage::Live { .. })
            })
    }

    pub(super) async fn staged_response_bytes(&self) -> HashMap<String, u64> {
        self.single_response_actions
            .lock()
            .await
            .iter()
            .filter_map(|(key, state)| match state.storage {
                SingleResponseStorage::Staged { received } => Some((key.clone(), received)),
                SingleResponseStorage::Live { .. } => None,
            })
            .collect()
    }

    pub(super) async fn retry_inactive_single_response(&self, key: &str) {
        let _update = self.update_key_raw(key).await;
        if let Err(error) = self.retry_inactive_single_response_locked(key).await {
            log::warn!("Could not retry abandoned video response: {error:#}");
        }
    }

    async fn retry_inactive_single_response_locked(&self, key: &str) -> Result<()> {
        let inactive = self
            .single_response_actions
            .lock()
            .await
            .get(key)
            .is_some_and(|state| !state.owner.is_active());
        if inactive {
            self.cancel_single_response(key).await?;
        }
        Ok(())
    }
}

impl ResponseOwner {
    pub(super) fn as_ref(&self) -> ResponseOwnerRef<'_> {
        match self {
            Self::Legacy(id) => ResponseOwnerRef::Legacy(*id),
            Self::Granted(action) => ResponseOwnerRef::Granted(action),
        }
    }
}

pub(super) async fn save_binding(
    store: &PartialRangeStore,
    binding: &RepresentationBinding,
) -> Result<()> {
    representation_disk::save(
        &store.paths.representation(binding.post().as_str()),
        binding.representation().fingerprint(),
    )
    .await
}
