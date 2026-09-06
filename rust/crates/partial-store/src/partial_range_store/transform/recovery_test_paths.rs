use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn temp_root() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("valid test fixture")
        .as_nanos();
    let parent = std::env::var_os("CARGO_TARGET_DIR")
        .map(|target| PathBuf::from(target).join("test-stores/partial-store"))
        .unwrap_or_else(std::env::temp_dir);
    parent.join(format!(
        "ghostr-transform-recovery-{}-{stamp}",
        std::process::id()
    ))
}
