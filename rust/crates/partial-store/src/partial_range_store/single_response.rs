use super::PartialRangeStore;
use crate::partial_range_disk as disk;
use crate::partial_range_manifest::RangeManifest;
use crate::partial_range_representation_disk as representation_disk;
use anyhow::{ensure, Result};
use ghostr_engine::adaptive::WholeBodyContract;
use ghostr_engine::representation::{RepresentationBinding, TransferIdentity};

mod commit;
mod continuity;
mod finish;
mod lifecycle;
mod open;
mod opening;
mod session;
mod staged;
mod state;
#[cfg(any(test, feature = "test"))]
mod test_support;
mod transaction;
mod write;
pub(in crate::partial_range_store) use commit::{CommitPhase, CommitTarget, ResponseCommit};
use continuity::StagedCommitPolicy;
pub(in crate::partial_range_store) use session::SessionResponse;
pub(super) use state::{
    accepted_total, ResponseOwner, ResponseOwnerRef, SingleResponseAuthority, SingleResponseState,
    SingleResponseStorage,
};
pub(in crate::partial_range_store) use transaction::rollback_commit;

impl PartialRangeStore {
    async fn prepare_live_exact_response(
        &self,
        identity: &TransferIdentity,
        binding: &RepresentationBinding,
        contract: WholeBodyContract,
        storage: SingleResponseStorage,
    ) -> Result<()> {
        let WholeBodyContract::Exact { expected_bytes } = contract else {
            return Ok(());
        };
        if !matches!(storage, SingleResponseStorage::Live { .. }) {
            return Ok(());
        }
        self.selected()
            .insert(binding.post().as_str().to_owned(), identity.clone());
        self.install_provisional_total(binding, expected_bytes)
            .await
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
        Ok(if can_stream_live {
            SingleResponseStorage::Live { started: false }
        } else {
            SingleResponseStorage::Staged { received: 0 }
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
