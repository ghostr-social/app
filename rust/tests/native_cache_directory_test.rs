use rust_lib_ghostr::video::native_cache::prepare_native_cache_directory;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn clears_stale_native_downloads_before_startup() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!("ghostr-cache-{nonce}"));
    let nested = directory.join("nested");
    fs::create_dir_all(&nested).expect("create fixture");
    fs::write(nested.join("stale.mp4"), b"stale").expect("write fixture");

    prepare_native_cache_directory(&directory).expect("prepare cache");

    assert!(directory.is_dir());
    assert_eq!(fs::read_dir(&directory).expect("read cache").count(), 0);
    fs::remove_dir(&directory).expect("remove fixture");
}
