use super::{recovery_result, PolicyRecovery};
use crate::partial_range_disk as disk;
use crate::partial_range_paths::StorePaths;
use crate::partial_range_store::policy_eviction::transaction;
use crate::partial_range_store::policy_intent;
use anyhow::Result;

pub(super) async fn recover(
    paths: &StorePaths,
    key: &str,
    retained_bytes: u64,
) -> Result<PolicyRecovery> {
    let usable = canonical_is_old(paths, key, retained_bytes).await;
    if !usable {
        return Ok(PolicyRecovery {
            usable: false,
            cleanup_debt: Some(transaction::extra_payload_bytes(paths, key).await),
        });
    }
    let cleanup = match transaction::remove_uncommitted_payload(paths, key).await {
        Ok(()) => policy_intent::remove_authority(paths, key).await,
        Err(error) => Err(error),
    };
    recovery_result(paths, key, usable, cleanup.err()).await
}

pub(super) async fn recover_malformed(
    paths: &StorePaths,
    key: &str,
    error: anyhow::Error,
) -> Result<PolicyRecovery> {
    let staged = policy_intent::payload_bytes(paths, key).await;
    let usable = staged > 0 && canonical_is_old(paths, key, staged).await;
    log::warn!("Video store found malformed policy intent for {key}: {error:#}");
    if !usable {
        return Ok(PolicyRecovery {
            usable: false,
            cleanup_debt: Some(transaction::extra_payload_bytes(paths, key).await),
        });
    }
    let cleanup = retire(paths, key).await;
    recovery_result(paths, key, usable, cleanup.err().or(Some(error))).await
}

async fn canonical_is_old(paths: &StorePaths, key: &str, retained: u64) -> bool {
    let Ok(manifest) = disk::load_manifest(&paths.manifest(key)).await else {
        return false;
    };
    manifest.covered_bytes() > retained && transaction::canonical_pair_is_valid(paths, key).await
}

async fn retire(paths: &StorePaths, key: &str) -> Result<()> {
    policy_intent::remove_authority(paths, key).await?;
    transaction::remove_uncommitted_payload(paths, key).await
}
