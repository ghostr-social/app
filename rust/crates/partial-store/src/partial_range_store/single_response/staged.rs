use super::{PartialRangeStore, SingleResponseState, SingleResponseStorage};
use crate::partial_range_disk as disk;
use crate::partial_range_manifest::RangeManifest;
use crate::partial_range_store::cleanup_debt::CleanupScope;
use crate::partial_range_store::replacement_cleanup;
use crate::partial_range_store::StoreAction;
use anyhow::{ensure, Context, Result};
use ghostr_engine::representation::{RepresentationBinding, TransferIdentity};
use std::path::Path;

mod manifest;

impl PartialRangeStore {
    pub(super) async fn write_staged_single_response(
        &self,
        identity: &TransferIdentity,
        state: SingleResponseState,
        reservation: Option<&StoreAction>,
        offset: u64,
        bytes: &[u8],
    ) -> Result<bool> {
        let SingleResponseStorage::Staged { received } = state.storage else {
            unreachable!("staged writer requires staged state")
        };
        ensure!(
            offset == received,
            "single response writes must be sequential"
        );
        let end = offset
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| anyhow::anyhow!("single response length overflows"))?;
        ensure!(
            end <= state.contract.maximum_bytes(),
            "response exceeds length"
        );
        let key = identity.post().as_str();
        let mut entries = self.entries.lock().await;
        let reserved = match reservation {
            Some(action) => self.consume_action(action, bytes.len() as u64).await?,
            None => 0,
        };
        let result = self
            .write_staged_bytes(&mut entries, key, offset, bytes)
            .await;
        if let (Some(action), Err(_)) = (reservation, &result) {
            self.restore_action(action, reserved).await;
        }
        result?;
        drop(entries);
        self.record_staged_progress(key, &state.owner, end).await;
        Ok(true)
    }

    async fn write_staged_bytes(
        &self,
        entries: &mut crate::partial_range_store::Entries,
        key: &str,
        offset: u64,
        bytes: &[u8],
    ) -> Result<()> {
        self.make_room(entries, key, bytes.len() as u64).await?;
        disk::write_at(&self.paths.single_response(key), offset, bytes).await?;
        self.credit(bytes.len() as u64).await;
        self.capacity.spent(bytes.len() as u64).await;
        Ok(())
    }

    pub(super) async fn remove_staged_single_response(
        &self,
        state: &SingleResponseState,
    ) -> Result<()> {
        let key = state.identity.post().as_str();
        let SingleResponseStorage::Staged { received } = state.storage else {
            return Ok(());
        };
        disk::remove_if_present(&self.paths.single_response(key)).await?;
        disk::remove_if_present(&self.paths.single_response_manifest(key)).await?;
        disk::remove_if_present(&self.paths.single_response_commit(key)).await?;
        if received > 0 {
            self.release(received).await;
        }
        Ok(())
    }

    async fn record_staged_progress(&self, key: &str, owner: &super::ResponseOwner, received: u64) {
        let mut actions = self.single_response_actions.lock().await;
        let Some(state) = actions
            .get_mut(key)
            .filter(|state| state.owner.matches(owner.as_ref()))
        else {
            return;
        };
        state.storage = SingleResponseStorage::Staged { received };
    }

    pub(super) async fn commit_staged_single_response(
        &self,
        binding: &RepresentationBinding,
        total: u64,
    ) -> Result<()> {
        let key = binding.post().as_str();
        ensure!(
            disk::file_len(&self.paths.single_response(key)).await? == Some(total),
            "staged response length does not match its framing"
        );
        let manifest = manifest::complete(&self.paths.single_response(key), total).await?;
        disk::save_manifest(&self.paths.single_response_manifest(key), &manifest).await?;
        let mut entries = self.entries.lock().await;
        let old_accounted = self.entry(&mut entries, key).await?.accounted;
        if let Err(error) = self.publish_staged_response(key).await {
            self.rollback_staged_response(key).await;
            return Err(error);
        }
        self.record_staged_commit(&mut entries, key, manifest).await;
        let pending = self.take_sparse_response_bytes(key).await;
        self.finish_replacement_cleanup(key, old_accounted.saturating_add(pending))
            .await?;
        Ok(())
    }

    async fn publish_staged_response(&self, key: &str) -> Result<()> {
        replacement_cleanup::before_publish(&self.paths, key).await?;
        rename_if_present(&self.paths.partial(key), &self.paths.partial_staging(key)).await?;
        rename_if_present(&self.paths.manifest(key), &self.paths.manifest_backup(key)).await?;
        rename_if_present(
            &self.paths.generation(key),
            &self.paths.generation_backup(key),
        )
        .await?;
        tokio::fs::rename(self.paths.single_response(key), self.paths.partial(key))
            .await
            .context("publish complete single response")?;
        tokio::fs::rename(
            self.paths.single_response_manifest(key),
            self.paths.manifest(key),
        )
        .await
        .context("publish single response manifest")?;
        disk::write_marker(&self.paths.single_response_commit(key))
            .await
            .context("mark single response commit")
    }

    async fn rollback_staged_response(&self, key: &str) {
        let _ = disk::remove_if_present(&self.paths.single_response_commit(key)).await;
        restore(&self.paths.partial_staging(key), &self.paths.partial(key)).await;
        restore(&self.paths.manifest_backup(key), &self.paths.manifest(key)).await;
        restore(
            &self.paths.generation_backup(key),
            &self.paths.generation(key),
        )
        .await;
    }

    async fn record_staged_commit(
        &self,
        entries: &mut crate::partial_range_store::Entries,
        key: &str,
        manifest: RangeManifest,
    ) {
        self.advance_content_revision(key).await;
        self.source_generations.lock().await.remove(key);
        let entry = entries.get_mut(key).expect("staged response entry");
        entry.manifest = manifest;
        entry.accounted = entry.manifest.covered_bytes();
        entry.completion = None;
        entry.touched = self.tick();
        self.changed.notify_waiters();
    }

    async fn finish_replacement_cleanup(&self, key: &str, charged: u64) -> Result<()> {
        if let Err(error) = replacement_cleanup::published(&self.paths, key).await {
            self.transfer_charged_cleanup_debt(key, CleanupScope::ReplacedCanonical, charged)
                .await?;
            log::warn!("Could not clean replaced video bytes for {key}: {error:#}");
            return Ok(());
        }
        self.release(charged).await;
        Ok(())
    }
}

async fn rename_if_present(source: &Path, target: &Path) -> Result<()> {
    if disk::file_len(source).await?.is_some() {
        tokio::fs::rename(source, target).await?;
    }
    Ok(())
}

async fn restore(backup: &Path, canonical: &Path) {
    if disk::file_len(backup).await.ok().flatten().is_none() {
        return;
    }
    let _ = disk::remove_if_present(canonical).await;
    let _ = tokio::fs::rename(backup, canonical).await;
}
