use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn temp_root() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "ghostr-transform-recovery-{}-{stamp}",
        std::process::id()
    ))
}
