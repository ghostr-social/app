use core::sync::atomic::{AtomicU64, Ordering};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn unique(prefix: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
    let process = std::process::id();
    let parent = std::env::var_os("CARGO_TARGET_DIR")
        .map(|target| PathBuf::from(target).join("test-stores/api"))
        .unwrap_or_else(std::env::temp_dir);
    parent.join(format!("{prefix}-{nonce}-{process}-{sequence}"))
}
