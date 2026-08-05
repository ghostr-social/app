mod support;

use std::fs::{FileTimes, OpenOptions};
use std::time::SystemTime;
use support::fixtures::trusted_media_client;
use support::http::unused_loopback_url;
use support::native_cache::{advertised_key, NativeCacheHarness, VIDEO_DIGEST, VIDEO_RESPONSE};

#[tokio::test]
async fn advertised_native_blob_is_rehashed_after_its_mtime_changes() {
    let harness = NativeCacheHarness::new("ghostr-cache-rehash");
    let key = advertised_key();
    let cached = harness
        .download(&key, VIDEO_RESPONSE, Some(VIDEO_DIGEST))
        .await;
    OpenOptions::new()
        .write(true)
        .open(&cached.path)
        .expect("open cached blob")
        .set_times(FileTimes::new().set_modified(SystemTime::UNIX_EPOCH))
        .expect("change modification time");
    let unavailable = unused_loopback_url().await;

    let reused = harness
        .cache
        .download(
            &trusted_media_client(),
            &key,
            &unavailable,
            Some(VIDEO_DIGEST),
        )
        .await
        .expect("rehash cached blob");

    assert_eq!(reused.path, cached.path);
    std::fs::remove_dir_all(harness.directory).expect("remove cache");
}
