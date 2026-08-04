use crate::video::native_cache_failure::permanent;
use crate::video::native_models::NativeVideoCacheKey;
use crate::video::native_partial_store::NativePartialStore;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Files the whole-file cache leaves behind. Its index is rebuilt from
/// nothing every run, so anything it wrote is unreachable and would
/// otherwise hold disk space for the life of the install.
const STALE_DOWNLOADS: [&str; 2] = ["mp4", "partial"];

/// Startup housekeeping for the cache directory. The directory itself
/// outlives the process: the progressive range store and the host model
/// live in it, and clearing them would throw away every byte the user
/// already paid for — device pass 3 measured the store at 8 KB after
/// every launch. Only the stale whole-file downloads are swept.
pub fn prepare_native_cache_directory(directory: &Path) -> Result<()> {
    fs::create_dir_all(directory).context("create native video cache")?;
    for item in fs::read_dir(directory).context("read native video cache")? {
        let path = item.context("read native video cache entry")?.path();
        if is_stale_download(&path) {
            fs::remove_file(&path).context("clear stale native download")?;
        }
    }
    Ok(())
}

fn is_stale_download(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| STALE_DOWNLOADS.contains(&extension))
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
