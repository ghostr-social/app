//! How much room the device actually has. A configured budget says what
//! the user is willing to spend; only the file system says what is there
//! to spend, and it changes while the app runs because other apps write
//! to the same volume.

use std::path::{Path, PathBuf};

/// Free space on the file system that holds a path, in bytes. `None`
/// means "cannot be measured" — callers then fall back to the budget.
pub trait FreeSpace: Send + Sync {
    fn available_bytes(&self, path: &Path) -> Option<u64>;
}

/// The device's own file system, measured with `statvfs(3)`.
pub struct SystemFreeSpace;

impl FreeSpace for SystemFreeSpace {
    fn available_bytes(&self, path: &Path) -> Option<u64> {
        measure(&nearest_existing(path)?)
    }
}

/// The store root does not exist until its first write, so the
/// measurement walks up to the closest directory that does exist.
fn nearest_existing(path: &Path) -> Option<PathBuf> {
    path.ancestors()
        .find(|candidate| candidate.exists())
        .map(Path::to_path_buf)
}

#[cfg(unix)]
fn measure(path: &Path) -> Option<u64> {
    use std::ffi::CString;
    use std::mem::MaybeUninit;
    use std::os::unix::ffi::OsStrExt;

    let path = CString::new(path.as_os_str().as_bytes()).ok()?;
    let mut stats = MaybeUninit::<libc::statvfs>::uninit();
    // SAFETY: `path` is a valid NUL-terminated C string and `stats` is a
    // correctly sized, writable `statvfs`, read back only after the call
    // reports success.
    let outcome = unsafe { libc::statvfs(path.as_ptr(), stats.as_mut_ptr()) };
    statvfs_available(outcome, || {
        let stats = unsafe { stats.assume_init() };
        stats.f_bavail.checked_mul(stats.f_frsize)
    })
}

#[cfg(unix)]
pub(crate) fn statvfs_available(
    outcome: libc::c_int,
    measured: impl FnOnce() -> Option<u64>,
) -> Option<u64> {
    if outcome != 0 {
        return None;
    }
    measured()
}

/// No portable measurement outside Unix; the budget then stands alone.
#[cfg(not(unix))]
fn measure(_path: &Path) -> Option<u64> {
    None
}
