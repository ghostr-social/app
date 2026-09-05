use super::super::{PartialRangeStore, SingleResponseState};
use anyhow::Result;
use ghostr_engine::adaptive::WholeBodyContract;
use ghostr_engine::evidence::EvidenceValidator;
use ghostr_engine::representation::{RepresentationBinding, SourceGeneration};

impl PartialRangeStore {
    pub(super) async fn retain_versioned_prefix(
        &self,
        binding: &RepresentationBinding,
        state: &SingleResponseState,
    ) -> Result<bool> {
        let Some(generation) = resumable_generation(state) else {
            return Ok(false);
        };
        let key = binding.post().as_str();
        if !self
            .has_incomplete_prefix(key, generation.total_bytes())
            .await?
        {
            return Ok(false);
        }
        let source = state.identity.source().as_str();
        self.persist_generation(key, binding, Some((source, generation.clone())))
            .await?;
        self.source_generations
            .lock()
            .await
            .insert(key.to_owned(), (source.to_owned(), generation));
        self.changed.notify_waiters();
        Ok(true)
    }

    async fn has_incomplete_prefix(&self, key: &str, total: u64) -> Result<bool> {
        let mut entries = self.entries.lock().await;
        let ranges = self.entry(&mut entries, key).await?.manifest.ranges();
        Ok(matches!(ranges.as_slice(), [range] if range.start == 0 && range.end < total))
    }
}

fn resumable_generation(state: &SingleResponseState) -> Option<SourceGeneration> {
    let WholeBodyContract::Exact { expected_bytes } = state.contract else {
        return None;
    };
    let key = state.authority.generation()?.key();
    let EvidenceValidator::StrongEtag(etag) = key.validator()? else {
        return None;
    };
    SourceGeneration::try_new(key.final_url(), etag, expected_bytes).ok()
}
