use crate::partial_range_store::cleanup_debt::CleanupScope;
use crate::partial_range_store::sparse_intent;
use crate::partial_range_store::PartialRangeStore;
use anyhow::Result;

impl PartialRangeStore {
    pub(super) async fn retire_stale_sparse_responses(&self, key: &str) -> Result<()> {
        let pending = self.take_sparse_response_bytes(key).await;
        match sparse_intent::remove(&self.paths, key).await {
            Ok(()) => {
                self.release(pending).await;
                Ok(())
            }
            Err(error) => {
                self.transfer_charged_cleanup_debt(key, CleanupScope::RetiredSparse, pending)
                    .await?;
                Err(error)
            }
        }
    }
}
