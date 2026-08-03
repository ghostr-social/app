use crate::video::native_cache::CachedVideo;
use crate::video::native_models::NativeVideoCacheKey;
use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::time::SystemTime;
use tokio::io::AsyncReadExt;

#[derive(Clone)]
pub struct NativeBlobSnapshot {
    pub key: NativeVideoCacheKey,
    pub modified: Option<SystemTime>,
    pub video: CachedVideo,
}

pub struct NativeBlobValidation {
    pub modified: Option<SystemTime>,
    pub valid: bool,
}

pub async fn validate_blob(snapshot: &NativeBlobSnapshot) -> Result<NativeBlobValidation> {
    let metadata = match tokio::fs::symlink_metadata(&snapshot.video.path).await {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(invalid()),
        Err(error) => return Err(error).context("inspect active native blob"),
    };
    if !metadata.file_type().is_file() || metadata.len() != snapshot.video.bytes {
        return Ok(invalid());
    }
    let modified = metadata.modified().ok();
    let valid = digest_is_valid(snapshot, modified).await?;
    Ok(NativeBlobValidation { modified, valid })
}

async fn digest_is_valid(
    snapshot: &NativeBlobSnapshot,
    modified: Option<SystemTime>,
) -> Result<bool> {
    let NativeVideoCacheKey::AdvertisedDigest(expected) = &snapshot.key else {
        return Ok(true);
    };
    if modified.is_some() && modified == snapshot.modified {
        return Ok(true);
    }
    Ok(sha256_file(&snapshot.video.path)
        .await?
        .eq_ignore_ascii_case(expected))
}

async fn sha256_file(path: &std::path::Path) -> Result<String> {
    let mut file = tokio::fs::File::open(path)
        .await
        .context("open native blob for digest validation")?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let bytes = file
            .read(&mut buffer)
            .await
            .context("read native blob for digest validation")?;
        if bytes == 0 {
            break;
        }
        digest.update(&buffer[..bytes]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn invalid() -> NativeBlobValidation {
    NativeBlobValidation {
        modified: None,
        valid: false,
    }
}

pub async fn remove_if_present(path: &std::path::Path) -> Result<()> {
    let metadata = match tokio::fs::symlink_metadata(path).await {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error).context("inspect native blob"),
    };
    if metadata.file_type().is_dir() {
        tokio::fs::remove_dir_all(path)
            .await
            .context("remove native blob")?;
    } else {
        tokio::fs::remove_file(path)
            .await
            .context("remove native blob")?;
    }
    Ok(())
}
