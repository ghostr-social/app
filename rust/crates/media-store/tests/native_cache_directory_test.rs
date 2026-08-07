//! Startup housekeeping must not throw away the engine's own storage.
//! Only the whole-file downloads are stale — nothing rebuilds their
//! bookkeeping — while the progressive store and the host model in the
//! same directory have to outlive the process.

use ghostr_media_store::native_cache::prepare_native_cache_directory;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn keeps_the_progressive_store_and_clears_stale_downloads() {
    let directory = fixture_directory();
    let progressive = directory.join("progressive");
    fs::create_dir_all(&progressive).expect("create fixture");
    fs::write(progressive.join("clip.part"), b"prefetched").expect("partial video");
    fs::write(progressive.join("clip.ranges.json"), b"{}").expect("range manifest");
    fs::write(directory.join("host_stats.json"), b"{}").expect("host model");
    fs::write(directory.join("stale.mp4"), b"stale").expect("stale download");
    fs::write(directory.join("stale.partial"), b"stale").expect("stale partial");

    prepare_native_cache_directory(&directory).expect("prepare cache");

    assert!(directory.is_dir());
    assert!(progressive.join("clip.part").exists(), "prefetched bytes");
    assert!(progressive.join("clip.ranges.json").exists(), "manifest");
    assert!(directory.join("host_stats.json").exists(), "host model");
    assert!(!directory.join("stale.mp4").exists(), "stale download");
    assert!(!directory.join("stale.partial").exists(), "stale partial");
    fs::remove_dir_all(&directory).expect("remove fixture");
}

#[test]
fn creates_the_cache_directory_when_it_is_missing() {
    let directory = fixture_directory();

    prepare_native_cache_directory(&directory).expect("prepare cache");

    assert!(directory.is_dir());
    fs::remove_dir_all(&directory).expect("remove fixture");
}

fn fixture_directory() -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!("ghostr-cache-{nonce}"))
}
