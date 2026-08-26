use super::integrity;
use crate::partial_range_disk as disk;
use crate::partial_range_manifest::RangeManifest;
use crate::partial_range_paths::StorePaths;
use crate::partial_range_store::policy_intent::{self, TransactionIntent};
use anyhow::{Context as _, Result};
use std::path::{Path, PathBuf};

pub(super) async fn publish(paths: &StorePaths, key: &str) -> Result<()> {
    move_old_pair(paths, key).await?;
    move_new_pair(paths, key).await?;
    ensure_canonical_pair(paths, key).await?;
    match policy_intent::remove_authority(paths, key).await {
        Ok(()) => Ok(()),
        Err(error) if !policy_intent::exists(paths, key).await? => {
            log::warn!("Policy commit sync was ambiguous for {key}: {error:#}");
            Ok(())
        }
        Err(error) => Err(error),
    }
}

pub(super) async fn rollback(
    paths: &StorePaths,
    key: &str,
    intent: &TransactionIntent,
) -> Result<()> {
    restore_backups(paths, key).await?;
    ensure_old_pair(paths, key, intent).await?;
    remove_uncommitted_payload(paths, key).await?;
    policy_intent::remove_authority(paths, key).await
}

async fn restore_backups(paths: &StorePaths, key: &str) -> Result<()> {
    restore_if_present(&paths.policy_data_backup(key), &paths.partial(key)).await?;
    restore_if_present(&paths.policy_manifest_backup(key), &paths.manifest(key)).await?;
    disk::sync_parent(&paths.partial(key)).await
}

pub(super) async fn cleanup_committed(paths: &StorePaths, key: &str) -> Result<()> {
    ensure_canonical_pair(paths, key).await?;
    disk::sync_parent(&paths.partial(key)).await?;
    disk::remove_if_present(&paths.policy_data_backup(key)).await?;
    disk::sync_parent(&paths.partial(key)).await?;
    remove_paths([
        paths.policy_manifest_backup(key),
        paths.policy_staging(key),
        paths.policy_manifest_staging(key),
        paths.policy_manifest_staging_temp(key),
        paths.policy_intent_staging(key),
    ])
    .await?;
    disk::sync_parent(&paths.partial(key)).await
}

pub(super) async fn remove_uncommitted_payload(paths: &StorePaths, key: &str) -> Result<()> {
    remove_paths([
        paths.policy_staging(key),
        paths.policy_manifest_staging(key),
        paths.policy_manifest_staging_temp(key),
        paths.policy_data_backup(key),
        paths.policy_manifest_backup(key),
        paths.policy_intent_staging(key),
    ])
    .await?;
    disk::sync_parent(&paths.partial(key)).await
}

pub(super) async fn has_backups(paths: &StorePaths, key: &str) -> Result<bool> {
    Ok(path_exists(&paths.policy_data_backup(key)).await?
        || path_exists(&paths.policy_manifest_backup(key)).await?)
}

pub(super) async fn backup_payload_bytes(paths: &StorePaths, key: &str) -> u64 {
    let length = backup_length(paths, key).await;
    match disk::load_manifest(&paths.policy_manifest_backup(key)).await {
        Ok(manifest) => manifest.covered_bytes().max(length),
        Err(_) => length,
    }
}

pub(super) async fn extra_payload_bytes(paths: &StorePaths, key: &str) -> u64 {
    let staged = policy_intent::payload_bytes(paths, key).await;
    staged.saturating_add(backup_payload_bytes(paths, key).await)
}

async fn ensure_canonical_pair(paths: &StorePaths, key: &str) -> Result<()> {
    let manifest = disk::load_manifest(&paths.manifest(key)).await?;
    anyhow::ensure!(
        integrity::manifest_is_valid(&paths.partial(key), &manifest).await?,
        "policy canonical pair failed checksum validation"
    );
    Ok(())
}

pub(super) async fn old_pair_is_valid(
    paths: &StorePaths,
    key: &str,
    intent: &TransactionIntent,
) -> bool {
    ensure_old_pair(paths, key, intent).await.is_ok()
}

pub(super) async fn canonical_pair_is_valid(paths: &StorePaths, key: &str) -> bool {
    ensure_canonical_pair(paths, key).await.is_ok()
}

async fn move_old_pair(paths: &StorePaths, key: &str) -> Result<()> {
    rename(&paths.partial(key), &paths.policy_data_backup(key)).await?;
    rename(&paths.manifest(key), &paths.policy_manifest_backup(key)).await?;
    disk::sync_parent(&paths.partial(key)).await
}

async fn move_new_pair(paths: &StorePaths, key: &str) -> Result<()> {
    rename(&paths.policy_staging(key), &paths.partial(key)).await?;
    rename(&paths.policy_manifest_staging(key), &paths.manifest(key)).await?;
    disk::sync_parent(&paths.partial(key)).await
}

async fn ensure_old_pair(paths: &StorePaths, key: &str, intent: &TransactionIntent) -> Result<()> {
    let bytes = tokio::fs::read(paths.manifest(key))
        .await
        .context("read restored policy manifest")?;
    anyhow::ensure!(
        disk::sha256_bytes(&bytes) == intent.old_manifest_sha256(),
        "restored policy manifest does not match intent"
    );
    let manifest = RangeManifest::from_json(core::str::from_utf8(&bytes)?)?;
    anyhow::ensure!(
        manifest.covered_bytes() == intent.old_accounted(),
        "restored policy accounting does not match intent"
    );
    ensure_canonical_pair(paths, key).await
}

async fn restore_if_present(backup: &Path, canonical: &Path) -> Result<()> {
    if !path_exists(backup).await? {
        return Ok(());
    }
    disk::remove_if_present(canonical).await?;
    tokio::fs::rename(backup, canonical)
        .await
        .context("restore policy eviction backup")
}

async fn rename(source: &Path, target: &Path) -> Result<()> {
    tokio::fs::rename(source, target)
        .await
        .context("move policy eviction component")
}

async fn remove_paths(paths: impl IntoIterator<Item = PathBuf>) -> Result<()> {
    for path in paths {
        disk::remove_if_present(&path).await?;
    }
    Ok(())
}

async fn path_exists(path: &Path) -> Result<bool> {
    match tokio::fs::symlink_metadata(path).await {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).context("inspect policy transaction component"),
    }
}

async fn backup_length(paths: &StorePaths, key: &str) -> u64 {
    disk::file_len(&paths.policy_data_backup(key))
        .await
        .ok()
        .flatten()
        .unwrap_or(0)
}
