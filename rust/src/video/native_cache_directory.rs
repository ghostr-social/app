use crate::video::native_cache_failure::permanent;
use crate::video::native_models::NativeVideoCacheKey;
use crate::video::native_partial_store::NativePartialStore;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub fn prepare_native_cache_directory(directory: &Path) -> Result<()> {
    if directory.exists() {
        fs::remove_dir_all(directory).context("clear native video cache")?;
    }
    fs::create_dir_all(directory).context("create native video cache")?;
    Ok(())
}

pub fn completed_path(directory: &Path, key: &NativeVideoCacheKey) -> Result<PathBuf> {
    let id = key
        .storage_id()
        .ok_or_else(|| permanent("native video cache identifier is invalid"))?;
    Ok(directory.join(format!("{id}.mp4")))
}

pub async fn install(
    partials: &NativePartialStore,
    partial: &Path,
    completed: &Path,
    bytes: u64,
) -> Result<()> {
    if let Err(error) = tokio::fs::rename(partial, completed).await {
        return Err(partials.cleanup_error(partial, bytes, error.into()).await);
    }
    Ok(())
}
