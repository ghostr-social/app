//! Exact policy-selected sparse eviction with crash-safe pair publication.

use super::PartialRangeStore;
use crate::partial_range_store::ContentRevision;
use anyhow::{ensure, Result};
use core::ops::Range;

mod integrity;
mod outcome;
mod plan;
mod preparation;
pub(super) mod recovery;
mod tail;
#[cfg(any(test, feature = "test"))]
mod test_support;
mod transaction;

pub use outcome::EvictionOutcome;

impl PartialRangeStore {
    /// # Errors
    ///
    /// Returns an error when revision validation or durable eviction cannot be completed.
    pub async fn evict_ranges_if_current(
        &self,
        key: &str,
        ranges: &[Range<u64>],
        revision: ContentRevision,
    ) -> Result<EvictionOutcome> {
        self.evict_ranges_at_revision(key, ranges, Some(revision))
            .await
    }

    async fn evict_ranges_at_revision(
        &self,
        key: &str,
        ranges: &[Range<u64>],
        revision: Option<ContentRevision>,
    ) -> Result<EvictionOutcome> {
        if ranges.is_empty() {
            return Ok(EvictionOutcome::default());
        }
        let _update = self.update_key_raw(key).await;
        self.retry_cleanup_debt_locked(key).await?;
        let Some(lease) = self.leases.try_acquire_unheld(key) else {
            return Ok(EvictionOutcome::default());
        };
        if self.reserved_keys().await.contains(key) {
            return Ok(EvictionOutcome::default());
        }
        let Some(plan) = self.policy_plan(key, ranges, revision).await? else {
            return Ok(EvictionOutcome::default());
        };
        Box::pin(self.apply_policy_plan(key, plan, &lease)).await
    }

    async fn policy_plan(
        &self,
        key: &str,
        ranges: &[Range<u64>],
        revision: Option<ContentRevision>,
    ) -> Result<Option<plan::EvictionPlan>> {
        let mut entries = self.entries.lock().await;
        if let Some(expected) = revision {
            if expected != self.current_content_revision(key).await {
                return Ok(None);
            }
        }
        let entry = self.entry(&mut entries, key).await?;
        Ok(Some(plan::build(entry, ranges)))
    }

    async fn apply_policy_plan(
        &self,
        key: &str,
        plan: plan::EvictionPlan,
        lease: &super::leases::StoreLease,
    ) -> Result<EvictionOutcome> {
        if plan.outcome.freed_bytes() == 0 {
            return Ok(plan.outcome);
        }
        if plan.completed || plan.retained.covered_bytes() == 0 {
            return self.evict_entire(key, &plan, lease).await;
        }
        if let Err(error) = Box::pin(self.prepare_and_publish(key, &plan, lease)).await {
            self.recover_after_policy_error(key, &error).await;
            return Err(error);
        }
        Box::pin(self.recover_policy_transaction_locked(key)).await?;
        Ok(plan.outcome)
    }

    async fn evict_entire(
        &self,
        key: &str,
        plan: &plan::EvictionPlan,
        lease: &super::leases::StoreLease,
    ) -> Result<EvictionOutcome> {
        ensure!(
            plan.outcome.freed_bytes() == plan.accounted,
            "cannot split a finalized video"
        );
        ensure!(lease.is_exclusive(), "policy eviction acquired by a reader");
        let mut entries = self.entries.lock().await;
        self.discard(&mut entries, key).await?;
        self.changed.notify_waiters();
        Ok(plan.outcome.clone())
    }

    async fn recover_after_policy_error(&self, key: &str, cause: &anyhow::Error) {
        if let Err(error) = self.recover_policy_transaction_locked(key).await {
            log::warn!("Policy transaction recovery failed after {cause:#}: {error:#}");
        }
    }
}
