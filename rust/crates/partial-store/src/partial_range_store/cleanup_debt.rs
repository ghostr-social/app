use super::{PartialRangeStore, StoreAction};
use anyhow::{ensure, Result};
use std::collections::HashMap;

mod isolated;
mod policy;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) enum CleanupScope {
    CanonicalDirty,
    PolicyStagingOnly,
    PolicyTransaction,
    ReplacedCanonical,
    RetiredSparse,
    StagingOnly,
}

pub(super) struct CleanupDebt {
    bytes: u64,
    owner: Option<StoreAction>,
}

impl PartialRangeStore {
    pub(super) async fn record_cleanup_debt(
        &self,
        key: &str,
        scope: CleanupScope,
        owner: Option<StoreAction>,
        bytes: u64,
    ) -> Result<()> {
        self.capacity.spent(bytes).await;
        let mut used = self.used_bytes.lock().await;
        let mut debts = self.cleanup_debts.lock().await;
        let debt = debts
            .entry((key.to_owned(), scope))
            .or_insert_with(|| CleanupDebt {
                bytes: 0,
                owner: owner.clone(),
            });
        ensure!(
            same_owner(debt.owner.as_ref(), owner.as_ref()),
            "cleanup debt owner changed"
        );
        debt.bytes = debt.bytes.saturating_add(bytes);
        *used = used.saturating_add(bytes);
        Ok(())
    }

    pub(super) async fn transfer_charged_cleanup_debt(
        &self,
        key: &str,
        scope: CleanupScope,
        bytes: u64,
    ) -> Result<()> {
        let mut debts = self.cleanup_debts.lock().await;
        let debt = debts.entry((key.to_owned(), scope)).or_insert(CleanupDebt {
            bytes: 0,
            owner: None,
        });
        ensure!(debt.owner.is_none(), "cleanup debt has an active owner");
        debt.bytes = debt.bytes.saturating_add(bytes);
        Ok(())
    }

    pub(super) async fn retry_cleanup_debt(&self, key: &str) -> Result<()> {
        let _update = self.update_key_raw(key).await;
        self.retry_cleanup_debt_locked(key).await
    }

    pub(super) async fn retry_cleanup_debt_locked(&self, key: &str) -> Result<()> {
        if self
            .has_cleanup_scope(key, CleanupScope::PolicyTransaction)
            .await
        {
            self.recover_policy_transaction_locked(key).await?;
        }
        let scopes = self.retryable_cleanup_scopes(key).await?;
        if scopes.contains(&CleanupScope::CanonicalDirty) {
            let mut entries = self.entries.lock().await;
            self.discard(&mut entries, key).await?;
        } else {
            self.retry_isolated_cleanup(key, &scopes).await?;
        }
        ensure!(
            !self.has_cleanup_debt(key).await,
            "video cleanup is still active"
        );
        Ok(())
    }

    async fn retryable_cleanup_scopes(&self, key: &str) -> Result<Vec<CleanupScope>> {
        let debts = self.cleanup_debts.lock().await;
        let matching: Vec<_> = debts
            .iter()
            .filter(|((known, _), _)| known == key)
            .collect();
        ensure!(
            matching
                .iter()
                .all(|(_, debt)| debt.owner.as_ref().is_none_or(|owner| !owner.is_active())),
            "video cleanup is still active"
        );
        Ok(matching.into_iter().map(|((_, scope), _)| *scope).collect())
    }

    pub(super) async fn release_cleanup_debts(&self, key: &str) {
        let mut debts = self.cleanup_debts.lock().await;
        let owned: Vec<_> = debts
            .keys()
            .filter(|(known, _)| known == key)
            .cloned()
            .collect();
        let released = owned
            .into_iter()
            .filter_map(|owned| debts.remove(&owned))
            .map(|debt| debt.bytes)
            .sum();
        drop(debts);
        self.release(released).await;
    }

    pub(super) async fn cleanup_debt_bytes(&self) -> HashMap<String, u64> {
        let mut totals = HashMap::<String, u64>::new();
        for ((key, _), debt) in self.cleanup_debts.lock().await.iter() {
            *totals.entry(key.clone()).or_default() += debt.bytes;
        }
        totals
    }

    async fn has_cleanup_debt(&self, key: &str) -> bool {
        self.cleanup_debts
            .lock()
            .await
            .keys()
            .any(|(known, _)| known == key)
    }

    async fn has_cleanup_scope(&self, key: &str, scope: CleanupScope) -> bool {
        self.cleanup_debts
            .lock()
            .await
            .contains_key(&(key.to_owned(), scope))
    }
}

pub(super) type CleanupDebts = HashMap<(String, CleanupScope), CleanupDebt>;

fn same_owner(known: Option<&StoreAction>, seen: Option<&StoreAction>) -> bool {
    match (known, seen) {
        (Some(known), Some(seen)) => known.same_authority(seen),
        (None, None) => true,
        _ => false,
    }
}
