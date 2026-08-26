use super::PartialRangeStore;
use crate::partial_range_representation_disk as identity_disk;
use anyhow::Result;
use ghostr_engine::representation::TransferIdentity;

impl PartialRangeStore {
    /// Writes bytes for the transfer selected by a legacy test scenario.
    ///
    /// # Errors
    ///
    /// Returns an error when authority checks or persistence fail.
    pub async fn write_range_for_transfer_if_current(
        &self,
        identity: &TransferIdentity,
        offset: u64,
        bytes: &[u8],
    ) -> Result<bool> {
        let key = identity.post().as_str();
        self.retry_cleanup_debt(key).await?;
        if !self.transfer_is_current(identity).await {
            return Ok(false);
        }
        let mut entries = self.entries.lock().await;
        if !self.transfer_is_current(identity).await
            || self.policy_transaction_debt(key).await.is_some()
        {
            return Ok(false);
        }
        self.write_range_locked(&mut entries, key, offset, bytes)
            .await?;
        if self.transfer_is_current(identity).await {
            return Ok(true);
        }
        self.discard_stale_write(&mut entries, key).await?;
        Ok(false)
    }

    async fn discard_stale_write(
        &self,
        entries: &mut super::super::Entries,
        key: &str,
    ) -> Result<()> {
        self.discard_before_authority(entries, key).await?;
        let binding = self.representations.lock().await.get(key).cloned();
        if let Some(binding) = binding {
            identity_disk::save(
                &self.paths.representation(key),
                binding.representation().fingerprint(),
            )
            .await?;
        }
        Ok(())
    }
}
