use anyhow::{Context as _, Result};
use std::fs;
use std::path::Path;

/// Files the whole-file cache leaves behind. Its index is rebuilt from
/// nothing every run, so anything it wrote is unreachable and would
/// otherwise hold disk space for the life of the install.
const STALE_DOWNLOADS: [&str; 2] = ["mp4", "partial"];

/// Performs startup housekeeping for the cache directory.
///
/// The directory itself outlives the process: the progressive range store and the host model
/// live in it, and clearing them would throw away every byte the user
/// already paid for — device pass 3 measured the store at 8 KB after
/// every launch. Only the stale whole-file downloads are swept.
///
/// # Errors
///
/// Returns an error when the cache directory cannot be created, read, or cleaned.
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
