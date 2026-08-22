use crate::partial_range_completion::{self as completion, Completion};
use crate::partial_range_manifest::RangeManifest;
use crate::partial_range_paths::StorePaths;
use anyhow::{ensure, Context, Result};
use std::io::SeekFrom;
use std::ops::Range;
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

mod digest;
mod durable;

/// In-memory bookkeeping for one stored key, rebuilt lazily from disk.
/// `completion` is `None` while the key is still partial.
pub struct Entry {
    pub manifest: RangeManifest,
    pub accounted: u64,
    pub completion: Option<Completion>,
    /// Monotonic use counter driving eviction order; `0` until the key
    /// is read or written in this run, which makes untouched keys the
    /// first candidates.
    pub touched: u64,
}

impl Entry {
    fn partial(manifest: RangeManifest) -> Self {
        let accounted = manifest.covered_bytes();
        Self {
            manifest,
            accounted,
            completion: None,
            touched: 0,
        }
    }

    fn completed(len: u64, completion: Completion, manifest: RangeManifest) -> Result<Self> {
        ensure!(
            manifest.total_len() == Some(len),
            "completed length mismatch"
        );
        ensure!(manifest.is_complete(), "completed manifest is incomplete");
        Ok(Self {
            manifest,
            accounted: len,
            completion: Some(completion),
            touched: 0,
        })
    }
}

pub async fn load_entry(paths: &StorePaths, key: &str) -> Result<Entry> {
    if let Some(len) = file_len(&paths.completed(key)).await? {
        let completion = completion::recorded(&paths.verified(key)).await?;
        let manifest = load_manifest(&paths.manifest(key)).await?;
        return Entry::completed(len, completion, manifest);
    }
    Ok(Entry::partial(load_manifest(&paths.manifest(key)).await?))
}

pub async fn write_at(path: &Path, offset: u64, bytes: &[u8]) -> Result<()> {
    write_at_inner(path, offset, bytes, true).await
}

pub async fn write_at_unsynced(path: &Path, offset: u64, bytes: &[u8]) -> Result<()> {
    write_at_inner(path, offset, bytes, false).await
}

async fn write_at_inner(path: &Path, offset: u64, bytes: &[u8], sync: bool) -> Result<()> {
    ensure_parent(path).await?;
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(path)
        .await
        .context("open partial video file")?;
    file.seek(SeekFrom::Start(offset))
        .await
        .context("seek partial video file")?;
    file.write_all(bytes)
        .await
        .context("write partial video range")?;
    file.flush().await.context("flush partial video range")?;
    if sync {
        file.sync_data().await.context("sync partial video range")?;
    }
    Ok(())
}

pub async fn sync_file(path: &Path) -> Result<()> {
    tokio::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .await
        .context("open partial video for sync")?
        .sync_all()
        .await
        .context("sync partial video")
}

pub async fn save_durable(path: &Path, staging: &Path, bytes: &[u8]) -> Result<()> {
    durable::replace(staging, path, bytes).await
}

pub async fn remove_durable(path: &Path) -> Result<()> {
    remove_if_present(path).await?;
    sync_parent(path).await
}

pub async fn read_span(path: &Path, span: &Range<u64>) -> Result<Vec<u8>> {
    let len = span
        .end
        .checked_sub(span.start)
        .context("partial video range is reversed")?;
    let len = usize::try_from(len).context("partial video range exceeds memory")?;
    let mut file = tokio::fs::File::open(path)
        .await
        .context("open partial video file")?;
    file.seek(SeekFrom::Start(span.start))
        .await
        .context("seek partial video file")?;
    let mut buffer = Vec::new();
    buffer
        .try_reserve_exact(len)
        .context("reserve partial video range")?;
    buffer.resize(len, 0);
    file.read_exact(&mut buffer)
        .await
        .context("read partial video range")?;
    Ok(buffer)
}

pub async fn sha256_file(path: &Path) -> Result<String> {
    digest::file(path).await
}

pub async fn sha256_span(path: &Path, span: &Range<u64>) -> Result<String> {
    digest::span(path, span).await
}

pub fn sha256_bytes(bytes: &[u8]) -> String {
    digest::bytes(bytes)
}

pub async fn checksum_blocks(
    path: &Path,
    ranges: &[Range<u64>],
) -> Result<Vec<(Range<u64>, String)>> {
    let block = ghostr_engine::adaptive::REQUEST_SLICE_BYTES;
    let mut checksums = Vec::new();
    for range in ranges {
        let mut start = range.start;
        while start < range.end {
            let span = start..range.end.min(start.saturating_add(block));
            checksums.push((span.clone(), sha256_span(path, &span).await?));
            start = span.end;
        }
    }
    Ok(checksums)
}

/// Writes the zero-byte marker that records a verified completion.
pub async fn write_marker(path: &Path) -> Result<()> {
    ensure_parent(path).await?;
    tokio::fs::File::create(path)
        .await
        .context("write partial store verification marker")?
        .sync_all()
        .await
        .context("sync partial store verification marker")?;
    sync_parent(path).await
}

pub async fn load_manifest(path: &Path) -> Result<RangeManifest> {
    match tokio::fs::read_to_string(path).await {
        Ok(text) => RangeManifest::from_json(&text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(RangeManifest::default()),
        Err(error) => Err(error).context("read partial range manifest"),
    }
}

/// Staged then renamed, so a manifest is never half written. A staging
/// write that fails — a full disk is the usual reason — leaves nothing
/// behind and leaves the committed manifest exactly as it was.
pub async fn save_manifest(path: &Path, manifest: &RangeManifest) -> Result<()> {
    let staging = path.with_extension("json.tmp");
    durable::replace(&staging, path, manifest.to_json()?.as_bytes()).await
}

pub async fn sync_parent(path: &Path) -> Result<()> {
    let parent = path.parent().context("partial store path has no parent")?;
    durable::sync_directory(parent).await
}

pub async fn file_len(path: &Path) -> Result<Option<u64>> {
    match tokio::fs::symlink_metadata(path).await {
        Ok(metadata) => Ok(metadata.file_type().is_file().then_some(metadata.len())),
        Err(error) => metadata_failure(error),
    }
}

fn metadata_failure(error: std::io::Error) -> Result<Option<u64>> {
    if error.kind() == std::io::ErrorKind::NotFound {
        return Ok(None);
    }
    Err(error).context("inspect partial store file")
}

pub async fn remove_if_present(path: &Path) -> Result<()> {
    match tokio::fs::remove_file(path).await {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).context("remove partial store file"),
    }
}

async fn ensure_parent(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .context("create partial store directory")?;
    }
    Ok(())
}
