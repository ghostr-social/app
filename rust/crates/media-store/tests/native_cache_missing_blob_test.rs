mod cache_fixture;

use cache_fixture::video_cache_key;
use cache_fixture::{NativeCacheHarness, VIDEO_RESPONSE};

#[tokio::test]
async fn missing_native_blob_is_fetched_again_and_reaccounted() {
    let harness = NativeCacheHarness::new("ghostr-cache-missing-blob");
    let key = video_cache_key();
    let cached = harness.download(&key, VIDEO_RESPONSE, None).await;
    tokio::fs::remove_file(&cached.path)
        .await
        .expect("remove cached blob");

    let repaired = harness.download(&key, VIDEO_RESPONSE, None).await;

    assert_eq!(repaired.path, cached.path);
    assert_eq!(*harness.used_bytes.lock().await, 5);
    std::fs::remove_dir_all(harness.directory).expect("remove cache");
}
