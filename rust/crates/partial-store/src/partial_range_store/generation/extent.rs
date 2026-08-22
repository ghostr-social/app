use crate::partial_range_disk as disk;
use crate::partial_range_store::PartialRangeStore;
use anyhow::Result;

impl PartialRangeStore {
    pub(super) async fn install_generation_total(&self, key: &str, total: u64) -> Result<bool> {
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        if entry.completion.is_some() {
            return Ok(entry.manifest.total_len() == Some(total));
        }
        let mut manifest = entry.manifest.clone();
        if manifest.set_total_len(total).is_err() {
            return Ok(false);
        }
        disk::save_manifest(&self.paths.manifest(key), &manifest).await?;
        entry.manifest = manifest;
        self.changed.notify_waiters();
        Ok(true)
    }
}
