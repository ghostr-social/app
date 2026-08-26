use super::PartialRangeStore;
use anyhow::{bail, Result};

impl PartialRangeStore {
    pub(in crate::partial_range_store) async fn ensure_sparse_mutable(
        &self,
        key: &str,
    ) -> Result<()> {
        let mut entries = self.entries.lock().await;
        if self.entry(&mut entries, key).await?.completion.is_some() {
            bail!("cannot replace a finalized video");
        }
        Ok(())
    }
}
