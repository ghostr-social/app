use super::super::{save_binding, PartialRangeStore};
use crate::partial_range_disk as disk;
use anyhow::{bail, Result};
use ghostr_engine::representation::RepresentationBinding;

impl PartialRangeStore {
    pub(super) async fn finish_live_single_response(
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

    pub(super) async fn rollback_live_single_response(
        &self,
        binding: &RepresentationBinding,
    ) -> Result<()> {
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
}
