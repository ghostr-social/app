mod support;

use std::fs::{FileTimes, OpenOptions};
use std::time::SystemTime;
use support::native_cache::{advertised_key, NativeCacheHarness, VIDEO_DIGEST, VIDEO_RESPONSE};

#[tokio::test]
async fn corrupted_advertised_native_blob_is_removed_and_downloaded_again() {
    let harness = NativeCacheHarness::new("ghostr-cache-corrupt");
    let key = advertised_key();
    let cached = harness
        .download(&key, VIDEO_RESPONSE, Some(VIDEO_DIGEST))
        .await;
    tokio::fs::write(&cached.path, b"wrong")
        .await
        .expect("corrupt cached blob");
    OpenOptions::new()
        .write(true)
        .open(&cached.path)
        .expect("open corrupt blob")
        .set_times(FileTimes::new().set_modified(SystemTime::UNIX_EPOCH))
        .expect("change modification time");

    let repaired = harness
        .download(&key, VIDEO_RESPONSE, Some(VIDEO_DIGEST))
        .await;

    assert_eq!(repaired.path, cached.path);
    assert_eq!(
        tokio::fs::read(&repaired.path).await.expect("blob"),
        b"video"
    );
    assert_eq!(*harness.used_bytes.lock().await, 5);
    std::fs::remove_dir_all(harness.directory).expect("remove cache");
}
