use super::{record, TransformPublication};
use crate::partial_range_disk as disk;
use crate::partial_range_manifest::RangeManifest;
use crate::partial_range_paths::StorePaths;
use crate::partial_range_representation_disk as identity_disk;
use anyhow::{Context, Result};
use ghostr_engine::representation::RepresentationBinding;
use std::path::Path;

pub(super) struct Prepared {
    pub(super) binding: RepresentationBinding,
    pub(super) manifest: RangeManifest,
    bytes: u64,
}

impl Prepared {
    pub(super) const fn output_bytes(&self) -> u64 {
        self.bytes
    }
}

pub(super) async fn stage(
    paths: &StorePaths,
    key: &str,
    publication: TransformPublication,
) -> Result<Prepared> {
    discard_staging(paths, key).await?;
    let result = stage_publication(paths, key, publication).await;
    if result.is_err() {
        discard_staging(paths, key).await?;
    }
    result
}

async fn stage_publication(
    paths: &StorePaths,
    key: &str,
    publication: TransformPublication,
) -> Result<Prepared> {
    let transform = paths.transform(key);
    disk::write_at(&transform.data(), 0, &publication.output).await?;
    let manifest = complete_manifest(&transform.data(), publication.output.len() as u64).await?;
    disk::save_manifest(&transform.manifest(), &manifest).await?;
    let digest = disk::sha256_bytes(&publication.output);
    let binding = publication
        .fence
        .binding
        .derive_transform(publication.kind, &digest)
        .context("derive transformed representation identity")?;
    identity_disk::save(
        &transform.identity(),
        binding.representation().fingerprint(),
    )
    .await?;
    let record = record::TransformRecord::new(
        &publication.fence.binding,
        &binding,
        publication.kind,
        digest,
        publication.output.len() as u64,
    );
    record::save_staging(paths, key, &record).await?;
    Ok(Prepared {
        binding,
        manifest,
        bytes: publication.output.len() as u64,
    })
}

async fn complete_manifest(path: &Path, bytes: u64) -> Result<RangeManifest> {
    let mut manifest = RangeManifest::default();
    manifest.set_total_len(bytes)?;
    manifest.insert(0..bytes)?;
    let span = 0..bytes;
    for (span, checksum) in disk::checksum_blocks(path, std::slice::from_ref(&span)).await? {
        manifest.record_checksum(span, checksum)?;
    }
    Ok(manifest)
}

pub(super) async fn commit(paths: &StorePaths, key: &str) -> Result<()> {
    let transform = paths.transform(key);
    disk::write_marker(&transform.commit()).await?;
    backup(&paths.completed(key), &transform.data_backup()).await?;
    backup(&paths.manifest(key), &transform.manifest_backup()).await?;
    backup(&paths.representation(key), &transform.identity_backup()).await?;
    backup(&paths.verified(key), &transform.verified_backup()).await?;
    tokio::fs::rename(transform.data(), paths.completed(key))
        .await
        .context("publish transformed bytes")?;
    tokio::fs::rename(transform.manifest(), paths.manifest(key))
        .await
        .context("publish transformed manifest")?;
    tokio::fs::rename(transform.identity(), paths.representation(key))
        .await
        .context("publish transformed identity")?;
    tokio::fs::rename(transform.record_staging(), transform.record())
        .await
        .context("publish transform provenance")?;
    disk::sync_parent(&paths.completed(key)).await?;
    finish(paths, key).await
}

async fn backup(source: &Path, target: &Path) -> Result<()> {
    if disk::file_len(source).await?.is_some() {
        tokio::fs::rename(source, target).await?;
    }
    Ok(())
}

pub(super) async fn finish(paths: &StorePaths, key: &str) -> Result<()> {
    let transform = paths.transform(key);
    for backup in [
        transform.data_backup(),
        transform.manifest_backup(),
        transform.identity_backup(),
        transform.verified_backup(),
    ] {
        disk::remove_if_present(&backup).await?;
    }
    discard_staging(paths, key).await?;
    disk::remove_durable(&transform.commit()).await
}

pub(super) async fn rollback(paths: &StorePaths, key: &str) -> Result<()> {
    let transform = paths.transform(key);
    restore(&transform.data_backup(), &paths.completed(key)).await?;
    restore(&transform.manifest_backup(), &paths.manifest(key)).await?;
    restore(&transform.identity_backup(), &paths.representation(key)).await?;
    restore(&transform.verified_backup(), &paths.verified(key)).await?;
    disk::remove_if_present(&transform.record()).await?;
    discard_staging(paths, key).await?;
    disk::remove_durable(&transform.commit()).await
}

async fn restore(backup: &Path, canonical: &Path) -> Result<()> {
    if disk::file_len(backup).await?.is_none() {
        return Ok(());
    }
    disk::remove_if_present(canonical).await?;
    tokio::fs::rename(backup, canonical).await?;
    Ok(())
}

pub(super) async fn discard_staging(paths: &StorePaths, key: &str) -> Result<()> {
    let transform = paths.transform(key);
    for path in [
        transform.data(),
        transform.manifest(),
        transform.manifest().with_extension("json.tmp"),
        transform.identity(),
        transform.identity().with_extension("representation.tmp"),
        transform.record_staging(),
        transform.record_staging().with_extension("json.tmp"),
    ] {
        disk::remove_if_present(&path).await?;
    }
    Ok(())
}
