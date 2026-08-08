use crate::native_cache::CachedVideo;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn temp_directory(prefix: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{prefix}-{nonce}"));
    std::fs::create_dir_all(&path).expect("create test directory");
    path
}

pub(super) fn cached(path: &Path, bytes: u64) -> CachedVideo {
    CachedVideo {
        path: path.to_path_buf(),
        bytes,
        content_length: Some(bytes),
    }
}
