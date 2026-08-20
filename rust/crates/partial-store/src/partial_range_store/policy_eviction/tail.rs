use super::{integrity, transaction};
use crate::partial_range_disk as disk;
use crate::partial_range_manifest::RangeManifest;
use crate::partial_range_paths::StorePaths;
use crate::partial_range_store::policy_intent::{self, TailIntent};
use anyhow::{Context, Result};
use std::path::Path;

pub(super) async fn prepare(
    paths: &StorePaths,
    key: &str,
    source: &RangeManifest,
    retained: &RangeManifest,
    old_hash: String,
    tail_end: u64,
) -> Result<()> {
    let manifest =
        integrity::verified_manifest(&paths.partial(key), source, retained, tail_end).await?;
    let bytes = manifest.to_json()?;
    stage_manifest(paths, key, bytes.as_bytes()).await?;
    let intent = TailIntent::new(
        source.covered_bytes(),
        manifest.covered_bytes(),
        old_hash,
        disk::sha256_bytes(bytes.as_bytes()),
        tail_end,
    );
    policy_intent::save_tail(paths, key, &intent).await?;
    truncate(paths, key, tail_end).await?;
    publish_manifest(paths, key).await?;
    policy_intent::remove_authority(paths, key).await
}

pub(super) async fn recover(paths: &StorePaths, key: &str, intent: &TailIntent) -> Result<bool> {
    if valid_old(paths, key, intent).await? {
        cleanup(paths, key).await?;
        return Ok(true);
    }
    if valid_new(paths, &paths.manifest(key), key, intent).await? {
        finish_forward(paths, key, intent, false).await?;
        return Ok(true);
    }
    if valid_new(paths, &paths.policy_manifest_staging(key), key, intent).await? {
        finish_forward(paths, key, intent, true).await?;
        return Ok(true);
    }
    Ok(false)
}

async fn finish_forward(
    paths: &StorePaths,
    key: &str,
    intent: &TailIntent,
    publish: bool,
) -> Result<()> {
    truncate(paths, key, intent.tail_end()).await?;
    if publish {
        publish_manifest(paths, key).await?;
    }
    cleanup(paths, key).await
}

async fn cleanup(paths: &StorePaths, key: &str) -> Result<()> {
    transaction::remove_uncommitted_payload(paths, key).await?;
    policy_intent::remove_authority(paths, key).await
}

async fn valid_old(paths: &StorePaths, key: &str, intent: &TailIntent) -> Result<bool> {
    valid_manifest(
        &paths.partial(key),
        &paths.manifest(key),
        intent.old_manifest_sha256(),
        intent.old_accounted(),
        None,
    )
    .await
}

async fn valid_new(
    paths: &StorePaths,
    manifest: &Path,
    key: &str,
    intent: &TailIntent,
) -> Result<bool> {
    valid_manifest(
        &paths.partial(key),
        manifest,
        intent.new_manifest_sha256(),
        intent.new_accounted(),
        Some(intent.tail_end()),
    )
    .await
}

async fn valid_manifest(
    data: &Path,
    path: &Path,
    expected_hash: &str,
    expected_bytes: u64,
    expected_end: Option<u64>,
) -> Result<bool> {
    let Some((bytes, manifest)) = load_manifest(path).await? else {
        return Ok(false);
    };
    if disk::sha256_bytes(&bytes) != expected_hash
        || manifest.covered_bytes() != expected_bytes
        || expected_end
            .is_some_and(|end| manifest.ranges().last().map(|range| range.end) != Some(end))
    {
        return Ok(false);
    }
    let required = manifest.ranges().last().map_or(0, |range| range.end);
    if disk::file_len(data).await?.unwrap_or(0) < required {
        return Ok(false);
    }
    integrity::manifest_is_valid(data, &manifest).await
}

async fn load_manifest(path: &Path) -> Result<Option<(Vec<u8>, RangeManifest)>> {
    let bytes = match tokio::fs::read(path).await {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).context("read tail eviction manifest"),
    };
    let Ok(text) = std::str::from_utf8(&bytes) else {
        return Ok(None);
    };
    let Ok(manifest) = RangeManifest::from_json(text) else {
        return Ok(None);
    };
    Ok(Some((bytes, manifest)))
}

async fn stage_manifest(paths: &StorePaths, key: &str, bytes: &[u8]) -> Result<()> {
    disk::save_durable(
        &paths.policy_manifest_staging(key),
        &paths.policy_manifest_staging_temp(key),
        bytes,
    )
    .await
}

async fn truncate(paths: &StorePaths, key: &str, end: u64) -> Result<()> {
    let file = tokio::fs::OpenOptions::new()
        .write(true)
        .open(paths.partial(key))
        .await
        .context("open policy tail video")?;
    file.set_len(end)
        .await
        .context("truncate policy tail video")?;
    file.sync_all().await.context("sync policy tail video")
}

async fn publish_manifest(paths: &StorePaths, key: &str) -> Result<()> {
    tokio::fs::rename(paths.policy_manifest_staging(key), paths.manifest(key))
        .await
        .context("publish policy tail manifest")?;
    disk::sync_parent(&paths.manifest(key)).await
}
