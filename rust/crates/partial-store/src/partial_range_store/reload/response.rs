use crate::partial_range_disk as disk;
use crate::partial_range_paths::StorePaths;
use crate::partial_range_store::replacement_cleanup;
use crate::partial_range_store::sparse_intent;
use anyhow::{Context, Result};
use log::warn;
use std::path::Path;

#[derive(Default)]
pub(super) struct SingleResponseRecovery {
    pub(super) published: bool,
    pub(super) replacement_debt: Option<u64>,
    pub(super) staging_debt: Option<u64>,
}

pub(super) async fn recover(paths: &StorePaths, key: &str) -> Result<SingleResponseRecovery> {
    let committed = exists(&paths.single_response_commit(key)).await?;
    if committed && published_response_is_complete(paths, key).await? {
        return Ok(SingleResponseRecovery {
            published: true,
            replacement_debt: cleanup_replaced_transaction(paths, key).await,
            staging_debt: None,
        });
    }
    restore_if_present(&paths.partial_staging(key), &paths.partial(key)).await?;
    restore_if_present(&paths.manifest_backup(key), &paths.manifest(key)).await?;
    restore_if_present(&paths.generation_backup(key), &paths.generation(key)).await?;
    Ok(SingleResponseRecovery {
        staging_debt: cleanup_transaction(paths, key).await,
        ..SingleResponseRecovery::default()
    })
}

async fn cleanup_replaced_transaction(paths: &StorePaths, key: &str) -> Option<u64> {
    let bytes = replacement_payload_bytes(paths, key).await;
    let error = replacement_cleanup::published(paths, key).await.err()?;
    warn!("Video store could not finish replacement cleanup {key}: {error:#}");
    Some(bytes)
}

async fn published_response_is_complete(paths: &StorePaths, key: &str) -> Result<bool> {
    let entry = disk::load_entry(paths, key).await?;
    let Some(total) = entry.manifest.total_len() else {
        return Ok(false);
    };
    let stored = disk::file_len(&paths.partial(key)).await?.unwrap_or(0);
    Ok(entry.completion.is_none() && entry.manifest.is_complete() && stored == total)
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
    let sparse = sparse_intent::cleanup_bound(paths, key).await;
    response.saturating_add(backup.max(sparse))
}

async fn exists(path: &Path) -> Result<bool> {
    Ok(disk::file_len(path).await?.is_some())
}
