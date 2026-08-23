use super::{CommitPhase, CommitTarget, ResponseCommit};
use crate::partial_range_completion::{self as completion, Completion};
use crate::partial_range_disk as disk;
use crate::partial_range_paths::StorePaths;
use crate::partial_range_store::replacement_cleanup;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub(super) async fn publish(
    paths: &StorePaths,
    key: &str,
    record: &mut ResponseCommit,
) -> Result<()> {
    replacement_cleanup::before_publish(paths, key).await?;
    record.save_phase(paths, key, CommitPhase::Prepared).await?;
    backup(paths, key, record).await?;
    record.save_phase(paths, key, CommitPhase::BackedUp).await?;
    install(paths, key, record.target()).await?;
    record.save_phase(paths, key, CommitPhase::Committed).await
}

pub(in crate::partial_range_store) async fn rollback_commit(
    paths: &StorePaths,
    key: &str,
    record: &ResponseCommit,
    preserve_stage: bool,
) -> Result<()> {
    if record.phase() != CommitPhase::Prepared {
        rollback_install(paths, key, record.target(), preserve_stage).await?;
    }
    restore_backups(paths, key, record).await?;
    disk::sync_parent(&paths.partial(key)).await?;
    if preserve_stage {
        clear_record(paths, key).await
    } else {
        replacement_cleanup::unpublished(paths, key).await
    }
}

async fn backup(paths: &StorePaths, key: &str, record: &ResponseCommit) -> Result<()> {
    for (source, target) in backup_paths(paths, key, record) {
        move_if_present(&source, &target).await?;
    }
    disk::sync_parent(&paths.partial(key)).await
}

async fn install(paths: &StorePaths, key: &str, target: CommitTarget) -> Result<()> {
    let data = match target {
        CommitTarget::Partial => paths.partial(key),
        CommitTarget::Verified => paths.completed(key),
    };
    tokio::fs::rename(paths.single_response(key), data)
        .await
        .context("publish complete single response")?;
    tokio::fs::rename(paths.single_response_manifest(key), paths.manifest(key))
        .await
        .context("publish single response manifest")?;
    if target == CommitTarget::Verified {
        completion::record(&paths.verified(key), Completion::Verified).await?;
    }
    disk::sync_parent(&paths.partial(key)).await
}

async fn rollback_install(
    paths: &StorePaths,
    key: &str,
    target: CommitTarget,
    preserve: bool,
) -> Result<()> {
    let data = match target {
        CommitTarget::Partial => paths.partial(key),
        CommitTarget::Verified => paths.completed(key),
    };
    if preserve {
        move_if_present(&data, &paths.single_response(key)).await?;
        move_if_present(&paths.manifest(key), &paths.single_response_manifest(key)).await?;
    } else {
        disk::remove_if_present(&data).await?;
        disk::remove_if_present(&paths.manifest(key)).await?;
    }
    if target == CommitTarget::Verified {
        disk::remove_if_present(&paths.verified(key)).await?;
    }
    Ok(())
}

async fn restore_backups(paths: &StorePaths, key: &str, record: &ResponseCommit) -> Result<()> {
    for (canonical, backup) in backup_paths(paths, key, record) {
        if disk::file_len(&backup).await?.is_none() {
            continue;
        }
        disk::remove_if_present(&canonical).await?;
        tokio::fs::rename(backup, canonical).await?;
    }
    Ok(())
}

fn backup_paths(paths: &StorePaths, key: &str, record: &ResponseCommit) -> Vec<(PathBuf, PathBuf)> {
    let mut pairs = vec![
        (paths.partial(key), paths.partial_staging(key)),
        (paths.manifest(key), paths.manifest_backup(key)),
        (paths.generation(key), paths.generation_backup(key)),
        (paths.completed(key), paths.completed_backup(key)),
        (paths.verified(key), paths.verified_backup(key)),
    ];
    if record.retire_http() {
        pairs.push((
            paths.http_generation(key),
            paths.http_generation_backup(key),
        ));
    }
    pairs
}

async fn clear_record(paths: &StorePaths, key: &str) -> Result<()> {
    disk::remove_if_present(&paths.single_response_commit_staging(key)).await?;
    disk::remove_durable(&paths.single_response_commit(key)).await
}

async fn move_if_present(source: &Path, target: &Path) -> Result<()> {
    if disk::file_len(source).await?.is_some() {
        tokio::fs::rename(source, target).await?;
    }
    Ok(())
}
