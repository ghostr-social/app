use crate::video::partial_range_manifest::RangeManifest;
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::io::SeekFrom;
use std::ops::Range;
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

/// In-memory bookkeeping for one stored key, rebuilt lazily from disk.
pub struct Entry {
    pub manifest: RangeManifest,
    pub accounted: u64,
    pub completed: bool,
}

impl Entry {
    fn partial(manifest: RangeManifest) -> Self {
        let accounted = manifest.covered_bytes();
        Self {
            manifest,
            accounted,
            completed: false,
        }
    }

    fn completed(len: u64) -> Result<Self> {
        let mut manifest = RangeManifest::default();
        manifest.set_total_len(len)?;
        manifest.insert(0..len)?;
        Ok(Self {
            manifest,
            accounted: len,
            completed: true,
        })
    }
}

pub async fn load_entry(completed: &Path, manifest: &Path) -> Result<Entry> {
    if let Some(len) = file_len(completed).await? {
        return Entry::completed(len);
    }
    Ok(Entry::partial(load_manifest(manifest).await?))
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

pub async fn load_manifest(path: &Path) -> Result<RangeManifest> {
    match tokio::fs::read_to_string(path).await {
        Ok(text) => Ok(RangeManifest::from_json(&text).unwrap_or_default()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(RangeManifest::default()),
        Err(error) => Err(error).context("read partial range manifest"),
    }
}

pub async fn save_manifest(path: &Path, manifest: &RangeManifest) -> Result<()> {
    ensure_parent(path).await?;
    let staging = path.with_extension("json.tmp");
    tokio::fs::write(&staging, manifest.to_json())
        .await
        .context("write partial range manifest")?;
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
