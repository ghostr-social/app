//! Measures how much room the device actually has.
//!
//! A configured budget says what the user is willing to spend; only the file system says what is there
//! to spend, and it changes while the app runs because other apps write
//! to the same volume.

use std::path::{Path, PathBuf};

/// Free space on the file system that holds a path, in bytes. `None`
/// means "cannot be measured" — callers then fall back to the budget.
pub trait FreeSpace: Send + Sync {
    fn available_bytes(&self, path: &Path) -> Option<u64>;
}

/// The device's own file system, measured by the platform adapter.
pub struct SystemFreeSpace;

impl FreeSpace for SystemFreeSpace {
    fn available_bytes(&self, path: &Path) -> Option<u64> {
        measure(&nearest_existing(path)?)
    }
}

/// The store root does not exist until its first write, so the
/// measurement walks up to the closest directory that does exist.
fn nearest_existing(path: &Path) -> Option<PathBuf> {
    let anchored = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(path)
    };
    anchored
        .ancestors()
        .find(|candidate| candidate.exists())
        .map(Path::to_path_buf)
}

fn measure(path: &Path) -> Option<u64> {
    fs2::available_space(path).ok()
}
