use super::PartialRangeStore;
use ghostr_engine::representation::{HttpGenerationKey, TransferIdentity};

impl PartialRangeStore {
    pub(in crate::partial_range_store) async fn http_generation_preserves_bytes(
        &self,
        identity: &TransferIdentity,
    ) -> bool {
        self.http_generations
            .lock()
            .await
            .get(identity.post().as_str())
            .is_some_and(|state| {
                state.source == identity.source().as_str()
                    && state
                        .key
                        .as_ref()
                        .and_then(HttpGenerationKey::validator)
                        .is_some()
            })
    }
}
