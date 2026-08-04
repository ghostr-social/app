use crate::video::partial_range_completion::{self as completion, Completion};
use crate::video::partial_range_manifest::RangeManifest;
use crate::video::partial_range_paths::StorePaths;
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::io::SeekFrom;
use std::ops::Range;
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

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

    fn completed(len: u64, completion: Completion) -> Result<Self> {
        let mut manifest = RangeManifest::default();
        manifest.set_total_len(len)?;
        manifest.insert(0..len)?;
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
        return Entry::completed(len, completion);
    }
    Ok(Entry::partial(load_manifest(&paths.manifest(key)).await?))
}

pub async fn write_at(path: &Path, offset: u64, bytes: &[u8]) -> Result<()> {
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
    Ok(())
}

pub async fn read_span(path: &Path, span: &Range<u64>) -> Result<Vec<u8>> {
    let mut file = tokio::fs::File::open(path)
        .await
        .context("open partial video file")?;
    file.seek(SeekFrom::Start(span.start))
        .await
        .context("seek partial video file")?;
    let mut buffer = vec![0_u8; (span.end - span.start) as usize];
    file.read_exact(&mut buffer)
        .await
        .context("read partial video range")?;
    Ok(buffer)
}

pub async fn sha256_file(path: &Path) -> Result<String> {
    let mut file = tokio::fs::File::open(path)
        .await
        .context("open partial video for digest")?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let bytes = file
            .read(&mut buffer)
            .await
            .context("read partial video for digest")?;
        if bytes == 0 {
            break;
        }
        digest.update(&buffer[..bytes]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

/// Writes the zero-byte marker that records a verified completion.
pub async fn write_marker(path: &Path) -> Result<()> {
    ensure_parent(path).await?;
    tokio::fs::write(path, [])
        .await
        .context("write partial store verification marker")
}

pub async fn load_manifest(path: &Path) -> Result<RangeManifest> {
    match tokio::fs::read_to_string(path).await {
        Ok(text) => Ok(RangeManifest::from_json(&text).unwrap_or_default()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(RangeManifest::default()),
        Err(error) => Err(error).context("read partial range manifest"),
    }
}

/// Staged then renamed, so a manifest is never half written. A staging
/// write that fails — a full disk is the usual reason — leaves nothing
/// behind and leaves the committed manifest exactly as it was.
pub async fn save_manifest(path: &Path, manifest: &RangeManifest) -> Result<()> {
    ensure_parent(path).await?;
    let staging = path.with_extension("json.tmp");
    if let Err(error) = tokio::fs::write(&staging, manifest.to_json()).await {
        remove_if_present(&staging).await?;
        return Err(error).context("write partial range manifest");
    }
    tokio::fs::rename(&staging, path)
        .await
        .context("commit partial range manifest")
}

pub async fn file_len(path: &Path) -> Result<Option<u64>> {
    match tokio::fs::symlink_metadata(path).await {
        Ok(metadata) if metadata.file_type().is_file() => Ok(Some(metadata.len())),
        Ok(_) => Ok(None),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).context("inspect partial store file"),
    }
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
