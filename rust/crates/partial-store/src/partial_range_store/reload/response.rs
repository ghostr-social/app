use crate::partial_range_disk as disk;
use crate::partial_range_paths::StorePaths;
use crate::partial_range_store::replacement_cleanup;
use crate::partial_range_store::single_response::{
    rollback_commit, CommitPhase, CommitTarget, ResponseCommit,
};
use crate::partial_range_store::sparse_intent;
use anyhow::{Context as _, Result};
use log::warn;
use std::path::Path;

#[derive(Default)]
pub(super) struct SingleResponseRecovery {
    pub(super) published: bool,
    pub(super) replacement_debt: Option<u64>,
    pub(super) staging_debt: Option<u64>,
}

pub(super) async fn recover(paths: &StorePaths, key: &str) -> Result<SingleResponseRecovery> {
    if let Ok(Some(record)) = ResponseCommit::load(paths, key).await {
        return recover_record(paths, key, record).await;
    }
    let committed = exists(&paths.single_response_commit(key)).await?;
    if committed {
        rollback_legacy_commit(paths, key).await?;
        return Ok(SingleResponseRecovery::default());
    }
    restore_if_present(&paths.partial_staging(key), &paths.partial(key)).await?;
    restore_if_present(&paths.manifest_backup(key), &paths.manifest(key)).await?;
    restore_if_present(&paths.generation_backup(key), &paths.generation(key)).await?;
    restore_if_present(
        &paths.http_generation_backup(key),
        &paths.http_generation(key),
    )
    .await?;
    Ok(SingleResponseRecovery {
        staging_debt: cleanup_transaction(paths, key).await,
        ..SingleResponseRecovery::default()
    })
}

async fn recover_record(
    paths: &StorePaths,
    key: &str,
    record: ResponseCommit,
) -> Result<SingleResponseRecovery> {
    if record.phase() == CommitPhase::Committed
        && committed_response_matches(paths, key, &record).await?
    {
        return Ok(SingleResponseRecovery {
            published: true,
            replacement_debt: cleanup_replaced_transaction(paths, key).await,
            staging_debt: None,
        });
    }
    rollback_commit(paths, key, &record, false).await?;
    Ok(SingleResponseRecovery::default())
}

async fn committed_response_matches(
    paths: &StorePaths,
    key: &str,
    record: &ResponseCommit,
) -> Result<bool> {
    let path = match record.target() {
        CommitTarget::Partial => paths.partial(key),
        CommitTarget::Verified => paths.completed(key),
    };
    if disk::file_len(&path).await? != Some(record.total()) {
        return Ok(false);
    }
    let entry = disk::load_entry(paths, key).await?;
    let expected_completion = match record.target() {
        CommitTarget::Partial => entry.completion.is_none(),
        CommitTarget::Verified => {
            entry.completion == Some(crate::partial_range_completion::Completion::Verified)
        }
    };
    Ok(expected_completion
        && entry.manifest.is_complete()
        && disk::sha256_file(&path)
            .await?
            .eq_ignore_ascii_case(record.sha256()))
}

async fn cleanup_replaced_transaction(paths: &StorePaths, key: &str) -> Option<u64> {
    let bytes = replacement_payload_bytes(paths, key).await;
    let error = replacement_cleanup::published(paths, key).await.err()?;
    warn!("Video store could not finish replacement cleanup {key}: {error:#}");
    Some(bytes)
}

async fn rollback_legacy_commit(paths: &StorePaths, key: &str) -> Result<()> {
    disk::remove_if_present(&paths.partial(key)).await?;
    disk::remove_if_present(&paths.manifest(key)).await?;
    restore_if_present(&paths.partial_staging(key), &paths.partial(key)).await?;
    restore_if_present(&paths.manifest_backup(key), &paths.manifest(key)).await?;
    restore_if_present(&paths.generation_backup(key), &paths.generation(key)).await?;
    restore_if_present(
        &paths.http_generation_backup(key),
        &paths.http_generation(key),
    )
    .await?;
    disk::sync_parent(&paths.partial(key)).await?;
    remove_transaction_files(paths, key).await
}

async fn restore_if_present(backup: &Path, canonical: &Path) -> Result<()> {
    if !exists(backup).await? {
        return Ok(());
    }
    disk::remove_if_present(canonical).await?;
    tokio::fs::rename(backup, canonical)
        .await
        .context("restore interrupted single response")
}

pub(super) async fn remove_transaction_files(paths: &StorePaths, key: &str) -> Result<()> {
    replacement_cleanup::unpublished(paths, key).await
}

async fn cleanup_transaction(paths: &StorePaths, key: &str) -> Option<u64> {
    let error = remove_transaction_files(paths, key).await.err()?;
    warn!("Video store could not clear staged {key}: {error:#}");
    Some(staging_payload_bytes(paths, key).await)
}

async fn staging_payload_bytes(paths: &StorePaths, key: &str) -> u64 {
    let response = disk::file_len(&paths.single_response(key))
        .await
        .ok()
        .flatten()
        .unwrap_or(0);
    let backup = disk::file_len(&paths.partial_staging(key))
        .await
        .ok()
        .flatten()
        .unwrap_or(0);
    response.saturating_add(backup)
}

async fn replacement_payload_bytes(paths: &StorePaths, key: &str) -> u64 {
    let response = disk::file_len(&paths.single_response(key))
        .await
        .ok()
        .flatten()
        .unwrap_or(0);
    let backup = disk::file_len(&paths.partial_staging(key))
        .await
        .ok()
        .flatten()
        .unwrap_or(0);
    let completed = disk::file_len(&paths.completed_backup(key))
        .await
        .ok()
        .flatten()
        .unwrap_or(0);
    let sparse = sparse_intent::cleanup_bound(paths, key).await;
    response.saturating_add(backup.max(completed).max(sparse))
}

async fn exists(path: &Path) -> Result<bool> {
    Ok(disk::file_len(path).await?.is_some())
}
