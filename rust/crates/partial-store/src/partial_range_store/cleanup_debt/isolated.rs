use super::CleanupScope;
use crate::partial_range_store::policy_eviction::recovery;
use crate::partial_range_store::reload::remove_transaction_files;
use crate::partial_range_store::{replacement_cleanup, sparse_intent, PartialRangeStore};
use anyhow::{ensure, Result};

impl PartialRangeStore {
    pub(super) async fn retry_isolated_cleanup(
        &self,
        key: &str,
        scopes: &[CleanupScope],
    ) -> Result<()> {
        if scopes.contains(&CleanupScope::StagingOnly) {
            remove_transaction_files(&self.paths, key).await?;
            self.release_cleanup_debt(key, CleanupScope::StagingOnly)
                .await;
        }
        self.retry_policy_staging(key, scopes).await?;
        if scopes.contains(&CleanupScope::ReplacedCanonical) {
            replacement_cleanup::published(&self.paths, key).await?;
            self.release_cleanup_debt(key, CleanupScope::ReplacedCanonical)
                .await;
        }
        if scopes.contains(&CleanupScope::RetiredSparse) {
            sparse_intent::remove(&self.paths, key).await?;
            self.release_cleanup_debt(key, CleanupScope::RetiredSparse)
                .await;
        }
        Ok(())
    }

    async fn retry_policy_staging(&self, key: &str, scopes: &[CleanupScope]) -> Result<()> {
        if !scopes.contains(&CleanupScope::PolicyStagingOnly) {
            return Ok(());
        }
        let recovered = recovery::recover(&self.paths, key).await?;
        let remaining = recovered.cleanup_debt.unwrap_or(0);
        self.reconcile_cleanup_debt(key, CleanupScope::PolicyStagingOnly, remaining)
            .await;
        ensure!(recovered.usable, "policy scratch recovery failed");
        ensure!(recovered.cleanup_debt.is_none(), "policy scratch remains");
        self.release_cleanup_debt(key, CleanupScope::PolicyStagingOnly)
            .await;
        Ok(())
    }

    async fn reconcile_cleanup_debt(&self, key: &str, scope: CleanupScope, bytes: u64) {
        let previous = self
            .cleanup_debts
            .lock()
            .await
            .get_mut(&(key.to_owned(), scope))
            .map(|debt| std::mem::replace(&mut debt.bytes, bytes));
        let Some(previous) = previous else { return };
        if bytes > previous {
            let added = bytes - previous;
            self.capacity.spent(added).await;
            self.credit(added).await;
        } else {
            self.release(previous - bytes).await;
        }
    }

    async fn release_cleanup_debt(&self, key: &str, scope: CleanupScope) {
        let debt = self
            .cleanup_debts
            .lock()
            .await
            .remove(&(key.to_owned(), scope));
        if let Some(debt) = debt {
            self.release(debt.bytes).await;
        }
    }
}
