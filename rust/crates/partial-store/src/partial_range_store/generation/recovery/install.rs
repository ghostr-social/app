use super::PartialRangeStore;
use crate::partial_range_disk as range_disk;
use crate::partial_range_generation_disk::StoredGeneration;
use anyhow::Result;
use ghostr_engine::representation::RepresentationBinding;

enum GenerationRestore {
    Installed,
    Superseded,
    Incompatible,
}

impl PartialRangeStore {
    pub(super) async fn restore_valid_generation(
        &self,
        binding: &RepresentationBinding,
        stored: StoredGeneration,
    ) -> Result<()> {
        let key = binding.post().as_str();
        match self.install_restored_generation(key, stored).await? {
            GenerationRestore::Installed | GenerationRestore::Superseded => Ok(()),
            GenerationRestore::Incompatible if self.stored_is_complete(key).await? => {
                range_disk::remove_if_present(&self.paths.generation(key)).await
            }
            GenerationRestore::Incompatible => {
                self.replace_stored_generation(key, binding, None).await
            }
        }
    }

    async fn install_restored_generation(
        &self,
        key: &str,
        stored: StoredGeneration,
    ) -> Result<GenerationRestore> {
        let mut entries = self.entries.lock().await;
        let Some(entry) = entries.get_mut(key) else {
            return Ok(GenerationRestore::Superseded);
        };
        if entry.completion.is_some() {
            return Ok(GenerationRestore::Incompatible);
        }
        let mut manifest = entry.manifest.clone();
        if manifest
            .set_total_len(stored.generation.total_bytes())
            .is_err()
        {
            return Ok(GenerationRestore::Incompatible);
        }
        range_disk::save_manifest(&self.paths.manifest(key), &manifest).await?;
        entry.manifest = manifest;
        let mut generations = self.source_generations.lock().await;
        generations.insert(key.to_owned(), (stored.source, stored.generation));
        self.changed.notify_waiters();
        Ok(GenerationRestore::Installed)
    }
}
