use super::SparseResponseState;
use crate::partial_range_store::{PartialRangeStore, StoreAction};
use anyhow::{ensure, Result};

impl PartialRangeStore {
    pub(in crate::partial_range_store) async fn retry_inactive_sparse_responses(
        &self,
        key: &str,
    ) -> Result<()> {
        let inactive = self.inactive_sparse_responses(key).await;
        for state in inactive {
            let result = self
                .finish_sparse_response(&state.identity, &state.generation, &state.owner)
                .await;
            if result.is_err() {
                self.quarantine_failed_sparse_action(&state.owner).await;
            }
            result?;
        }
        ensure!(
            self.inactive_sparse_responses(key).await.is_empty(),
            "abandoned sparse response is still active"
        );
        Ok(())
    }

    pub(in crate::partial_range_store) async fn quarantine_failed_sparse_action(
        &self,
        action: &StoreAction,
    ) {
        let key = action.identity().post().as_str();
        let _update = self.update_key_raw(key).await;
        let Some(state) = self.exact_sparse_response(action).await else {
            return;
        };
        if state.committed {
            return;
        }
        let mut entries = self.entries.lock().await;
        if let Err(error) = self.discard(&mut entries, key).await {
            log::warn!(
                "Could not quarantine failed sparse action {}: {error:#}",
                action.id()
            );
        }
    }

    async fn inactive_sparse_responses(&self, key: &str) -> Vec<SparseResponseState> {
        self.sparse_response_actions
            .lock()
            .await
            .values()
            .filter(|state| state.identity.post().as_str() == key && !state.owner.is_active())
            .cloned()
            .collect()
    }

    async fn exact_sparse_response(&self, action: &StoreAction) -> Option<SparseResponseState> {
        self.sparse_response_actions
            .lock()
            .await
            .get(&action.id())
            .filter(|state| state.owner.same_authority(action))
            .cloned()
    }
}
