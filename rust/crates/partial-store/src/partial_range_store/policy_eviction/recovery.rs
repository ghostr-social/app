use super::{tail, transaction};
use crate::partial_range_disk as disk;
use crate::partial_range_paths::StorePaths;
use crate::partial_range_store::policy_intent::{self, PolicyIntent, TailIntent};
use crate::partial_range_store::PartialRangeStore;
use anyhow::{bail, Result};

mod legacy;

pub(in crate::partial_range_store) struct PolicyRecovery {
    pub(in crate::partial_range_store) usable: bool,
    pub(in crate::partial_range_store) cleanup_debt: Option<u64>,
}

pub(in crate::partial_range_store) async fn recover(
    paths: &StorePaths,
    key: &str,
) -> Result<PolicyRecovery> {
    if policy_intent::exists(paths, key).await? {
        return recover_with_intent(paths, key).await;
    }
    if transaction::has_backups(paths, key).await? {
        return recover_committed(paths, key).await;
    }
    recover_uncommitted_scratch(paths, key).await
}

pub(in crate::partial_range_store) async fn cleanup_payload_bytes(
    paths: &StorePaths,
    key: &str,
) -> u64 {
    transaction::extra_payload_bytes(paths, key).await
}

async fn recover_with_intent(paths: &StorePaths, key: &str) -> Result<PolicyRecovery> {
    match policy_intent::load(paths, key).await {
        Ok(Some(PolicyIntent::Tail(intent))) => recover_tail(paths, key, &intent).await,
        Ok(Some(PolicyIntent::Transaction(intent))) => {
            let cleanup = transaction::rollback(paths, key, &intent).await;
            let usable = transaction::old_pair_is_valid(paths, key, &intent).await;
            recovery_result(paths, key, usable, cleanup.err()).await
        }
        Ok(Some(PolicyIntent::Legacy { retained_bytes })) => {
            legacy::recover(paths, key, retained_bytes).await
        }
        Ok(None) if transaction::has_backups(paths, key).await? => {
            recover_committed(paths, key).await
        }
        Ok(None) => recover_uncommitted_scratch(paths, key).await,
        Err(error) => legacy::recover_malformed(paths, key, error).await,
    }
}

async fn recover_tail(
    paths: &StorePaths,
    key: &str,
    intent: &TailIntent,
) -> Result<PolicyRecovery> {
    match tail::recover(paths, key, intent).await {
        Ok(usable) => Ok(PolicyRecovery {
            usable,
            cleanup_debt: (!usable).then_some(0),
        }),
        Err(error) => {
            log::warn!("Video store could not recover tail eviction {key}: {error:#}");
            Ok(PolicyRecovery {
                usable: transaction::canonical_pair_is_valid(paths, key).await,
                cleanup_debt: Some(0),
            })
        }
    }
}

async fn recover_committed(paths: &StorePaths, key: &str) -> Result<PolicyRecovery> {
    let usable = transaction::canonical_pair_is_valid(paths, key).await;
    if !usable {
        return Ok(PolicyRecovery {
            usable: false,
            cleanup_debt: Some(transaction::backup_payload_bytes(paths, key).await),
        });
    }
    let cleanup = transaction::cleanup_committed(paths, key).await;
    let cleanup_debt = match cleanup {
        Ok(()) => None,
        Err(error) => {
            log::warn!("Video store could not clean policy backups for {key}: {error:#}");
            Some(transaction::backup_payload_bytes(paths, key).await)
        }
    };
    Ok(PolicyRecovery {
        usable,
        cleanup_debt,
    })
}

async fn recover_uncommitted_scratch(paths: &StorePaths, key: &str) -> Result<PolicyRecovery> {
    let cleanup = transaction::remove_uncommitted_payload(paths, key).await;
    recovery_result(paths, key, true, cleanup.err()).await
}

pub(super) async fn recovery_result(
    paths: &StorePaths,
    key: &str,
    usable: bool,
    error: Option<anyhow::Error>,
) -> Result<PolicyRecovery> {
    if let Some(error) = error {
        log::warn!("Video store could not recover policy transaction {key}: {error:#}");
        return Ok(PolicyRecovery {
            usable,
            cleanup_debt: Some(transaction::extra_payload_bytes(paths, key).await),
        });
    }
    Ok(PolicyRecovery {
        usable,
        cleanup_debt: None,
    })
}

impl PartialRangeStore {
    pub(in crate::partial_range_store) async fn recover_policy_transaction_locked(
        &self,
        key: &str,
    ) -> Result<()> {
        if self.policy_transaction_debt(key).await.is_none() {
            return Ok(());
        }
        let _lease = self.lease(key);
        drop(self.entries.lock().await);
        if self.policy_transaction_debt(key).await.is_none() {
            return Ok(());
        }
        let recovered = recover(&self.paths, key).await?;
        if !recovered.usable {
            let mut entries = self.entries.lock().await;
            self.discard(&mut entries, key).await?;
            bail!("policy transaction could not restore a valid object");
        }
        let loaded = disk::load_entry(&self.paths, key).await?;
        self.install_policy_recovery(key, loaded, recovered.cleanup_debt)
            .await;
        Ok(())
    }

    async fn install_policy_recovery(
        &self,
        key: &str,
        mut loaded: disk::Entry,
        cleanup_debt: Option<u64>,
    ) {
        let mut entries = self.entries.lock().await;
        let previous = entries.get(key).map_or(0, |entry| entry.accounted);
        let changed = entries
            .get(key)
            .is_none_or(|entry| entry.manifest != loaded.manifest);
        loaded.touched = entries
            .get(key)
            .map_or(loaded.touched, |entry| entry.touched);
        let current = loaded.accounted;
        entries.insert(key.to_owned(), loaded);
        drop(entries);
        self.relabel_policy_accounting(key, previous, current, cleanup_debt)
            .await;
        if changed {
            self.advance_content_revision(key).await;
            self.changed.notify_waiters();
        }
    }
}
