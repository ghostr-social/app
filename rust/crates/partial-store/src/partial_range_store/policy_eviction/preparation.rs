use super::{integrity, plan, tail, transaction};
use crate::partial_range_disk as disk;
use crate::partial_range_manifest::RangeManifest;
use crate::partial_range_store::cleanup_debt::CleanupScope;
use crate::partial_range_store::leases::StoreLease;
use crate::partial_range_store::policy_intent::{self, TransactionIntent};
use crate::partial_range_store::PartialRangeStore;
use anyhow::{ensure, Context as _, Result};

impl PartialRangeStore {
    pub(super) async fn prepare_and_publish(
        &self,
        key: &str,
        plan: &plan::EvictionPlan,
        lease: &StoreLease,
    ) -> Result<()> {
        let old_hash = self.stable_manifest_hash(key, &plan.source).await?;
        if let Some(end) = plan.tail_end() {
            self.mark_tail_transaction(key, plan).await?;
            ensure!(lease.is_exclusive(), "policy eviction acquired by a reader");
            return tail::prepare(
                &self.paths,
                key,
                &plan.source,
                &plan.retained,
                old_hash,
                end,
            )
            .await;
        }
        self.prepare_copy_transaction(key, plan, lease, old_hash)
            .await
    }

    async fn prepare_copy_transaction(
        &self,
        key: &str,
        plan: &plan::EvictionPlan,
        lease: &StoreLease,
        old_hash: String,
    ) -> Result<()> {
        self.precharge_policy_transaction(key, plan).await?;
        let intent =
            TransactionIntent::new(plan.accounted, plan.retained.covered_bytes(), old_hash);
        policy_intent::save(&self.paths, key, &intent).await?;
        self.stage_policy_pair(key, &plan.source, &plan.retained)
            .await?;
        ensure!(lease.is_exclusive(), "policy eviction acquired by a reader");
        transaction::publish(&self.paths, key).await
    }

    async fn stable_manifest_hash(&self, key: &str, expected: &RangeManifest) -> Result<String> {
        let bytes = tokio::fs::read(self.paths.manifest(key))
            .await
            .context("read policy source manifest")?;
        ensure!(
            RangeManifest::from_json(core::str::from_utf8(&bytes)?)? == *expected,
            "policy source manifest changed"
        );
        Ok(disk::sha256_bytes(&bytes))
    }

    async fn mark_tail_transaction(&self, key: &str, plan: &plan::EvictionPlan) -> Result<()> {
        let _capacity = self.capacity_updates.lock().await;
        let entries = self.entries.lock().await;
        plan::ensure_current(entries.get(key), plan)?;
        self.record_cleanup_debt(key, CleanupScope::PolicyTransaction, None, 0)
            .await
    }

    async fn precharge_policy_transaction(
        &self,
        key: &str,
        plan: &plan::EvictionPlan,
    ) -> Result<()> {
        let retained = plan.retained.covered_bytes();
        let _capacity = self.capacity_updates.lock().await;
        let entries = self.entries.lock().await;
        plan::ensure_current(entries.get(key), plan)?;
        self.require_headroom(retained).await?;
        self.record_cleanup_debt(key, CleanupScope::PolicyTransaction, None, retained)
            .await
    }

    async fn stage_policy_pair(
        &self,
        key: &str,
        source: &RangeManifest,
        retained: &RangeManifest,
    ) -> Result<()> {
        let staging = self.paths.policy_staging(key);
        let manifest =
            integrity::stage_verified(&self.paths.partial(key), &staging, source, retained).await?;
        let bytes = manifest.to_json()?;
        disk::save_durable(
            &self.paths.policy_manifest_staging(key),
            &self.paths.policy_manifest_staging_temp(key),
            bytes.as_bytes(),
        )
        .await
    }
}
