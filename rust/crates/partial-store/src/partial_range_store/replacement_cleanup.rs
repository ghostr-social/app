use crate::partial_range_disk as disk;
use crate::partial_range_paths::StorePaths;
use crate::partial_range_store::sparse_intent;
use anyhow::Result;
use std::path::PathBuf;

pub(super) async fn before_publish(paths: &StorePaths, key: &str) -> Result<()> {
    let cleanup = remove_paths([
        paths.partial_staging(key),
        paths.manifest_backup(key),
        paths.generation_backup(key),
        paths.http_generation_backup(key),
        paths.completed_backup(key),
        paths.verified_backup(key),
        paths.single_response_commit_staging(key),
    ])
    .await;
    cleanup?;
    remove_marker(paths, key).await
}

pub(super) async fn unpublished(paths: &StorePaths, key: &str) -> Result<()> {
    remove_transaction_payload(paths, key).await?;
    remove_marker(paths, key).await
}

pub(super) async fn published(paths: &StorePaths, key: &str) -> Result<()> {
    let transaction = remove_transaction_payload(paths, key).await;
    let sparse = sparse_intent::remove(paths, key).await;
    transaction.and(sparse)?;
    remove_marker(paths, key).await
}

async fn remove_transaction_payload(paths: &StorePaths, key: &str) -> Result<()> {
    let marker = paths.single_response_commit(key);
    remove_paths(
        paths
            .single_response_artifacts(key)
            .into_iter()
            .filter(|path| path != &marker),
    )
    .await
}

async fn remove_marker(paths: &StorePaths, key: &str) -> Result<()> {
    disk::remove_durable(&paths.single_response_commit(key)).await
}

async fn remove_paths(paths: impl IntoIterator<Item = PathBuf>) -> Result<()> {
    let mut failure = None;
    for path in paths {
        if let Err(error) = disk::remove_if_present(&path).await {
            failure.get_or_insert(error);
        }
    }
    failure.map_or(Ok(()), Err)
}
