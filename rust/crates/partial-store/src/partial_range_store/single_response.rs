use super::PartialRangeStore;
use crate::partial_range_disk as disk;
use crate::partial_range_manifest::RangeManifest;
use crate::partial_range_representation_disk as representation_disk;
use crate::partial_range_store::response::ResponseOpenResult;
use crate::partial_range_store::StoreAction;
use anyhow::{ensure, Result};
use ghostr_engine::adaptive::WholeBodyContract;
use ghostr_engine::representation::{RepresentationBinding, TransferIdentity};

mod commit;
mod finish;
mod lifecycle;
mod session;
mod staged;
mod state;
mod transaction;
mod write;
pub(in crate::partial_range_store) use commit::{CommitPhase, CommitTarget, ResponseCommit};
pub(in crate::partial_range_store) use session::SessionResponse;
pub(super) use state::{
    accepted_total, ResponseOwner, ResponseOwnerRef, SingleResponseAuthority, SingleResponseState,
    SingleResponseStorage,
};
pub(in crate::partial_range_store) use transaction::rollback_commit;

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
                SingleResponseAuthority::Legacy,
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
        authority: SingleResponseAuthority,
    ) -> Result<ResponseOpenResult> {
        ensure!(
            contract.maximum_bytes() > 0,
            "single response limit must be positive"
        );
        let binding = self.current_binding(identity).await?;
        let key = identity.post().as_str();
        self.retry_inactive_single_response_locked(key).await?;
        if self.session_response(key).await.is_some() {
            return Ok(ResponseOpenResult::RequiresIndependentObject);
        }
        if let Some(known) = self.single_response_actions.lock().await.get(key) {
            let same = known.owner.matches(owner.as_ref())
                && known.identity == *identity
                && known.contract == contract
                && known.authority == authority;
            return Ok(if same {
                ResponseOpenResult::Opened
            } else {
                ResponseOpenResult::RequiresIndependentObject
            });
        }
        let force_staged = matches!(&authority, SingleResponseAuthority::ActionScoped);
        let storage = self
            .single_response_storage(key, contract, force_staged)
            .await?;
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
                authority,
            },
        );
        Ok(ResponseOpenResult::Opened)
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
        force_staged: bool,
    ) -> Result<SingleResponseStorage> {
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        ensure!(
            force_staged || entry.completion.is_none(),
            "cannot replace a finalized video"
        );
        let has_sparse_generation = self.source_generations.lock().await.contains_key(key);
        let can_stream_live = entry.accounted == 0
            && !has_sparse_generation
            && !force_staged
            && matches!(contract, WholeBodyContract::Exact { .. });
        Ok(match can_stream_live {
            true => SingleResponseStorage::Live { started: false },
            false => SingleResponseStorage::Staged { received: 0 },
        })
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
