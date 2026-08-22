use super::{CleanupDebt, CleanupScope};
use crate::partial_range_store::PartialRangeStore;

impl PartialRangeStore {
    pub(in crate::partial_range_store) async fn policy_transaction_debt(
        &self,
        key: &str,
    ) -> Option<u64> {
        self.cleanup_debts
            .lock()
            .await
            .get(&(key.to_owned(), CleanupScope::PolicyTransaction))
            .map(|debt| debt.bytes)
    }

    pub(in crate::partial_range_store) async fn relabel_policy_accounting(
        &self,
        key: &str,
        previous_entry: u64,
        current_entry: u64,
        current_debt: Option<u64>,
    ) {
        let previous_debt = self.policy_transaction_debt(key).await.unwrap_or(0);
        self.replace_policy_debt(key, current_debt).await;
        let previous = previous_entry.saturating_add(previous_debt);
        let current = current_entry.saturating_add(current_debt.unwrap_or(0));
        self.adjust_accounted_total(previous, current).await;
    }

    async fn replace_policy_debt(&self, key: &str, bytes: Option<u64>) {
        let mut debts = self.cleanup_debts.lock().await;
        let owned = (key.to_owned(), CleanupScope::PolicyTransaction);
        match bytes {
            Some(bytes) => {
                debts.insert(owned, CleanupDebt { bytes, owner: None });
            }
            None => {
                debts.remove(&owned);
            }
        }
    }

    async fn adjust_accounted_total(&self, previous: u64, current: u64) {
        if current > previous {
            let added = current - previous;
            self.credit(added).await;
            self.capacity.spent(added).await;
        } else {
            self.release(previous - current).await;
        }
    }
}
