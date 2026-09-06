use core::sync::atomic::{AtomicU64, Ordering};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// A directory no other caller holds. The clock alone cannot promise
/// that: it repeats a nanosecond reading often enough that two fixtures
/// built in the same instant would share a root, so the process and a
/// per-call counter carry the uniqueness and the reading only separates
/// this run from an earlier one that left a directory behind.
pub(super) fn temp_root(prefix: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
    let process = std::process::id();
    store_artifacts().join(format!("{prefix}-{nonce}-{process}-{sequence}"))
}

fn store_artifacts() -> PathBuf {
    std::env::var_os("CARGO_TARGET_DIR")
        .map(|target| PathBuf::from(target).join("test-stores/partial-store"))
        .unwrap_or_else(std::env::temp_dir)
}

pub(super) fn discard(root: &Path) {
    if root.exists() {
        std::fs::remove_dir_all(root).expect("remove store");
    }
}
