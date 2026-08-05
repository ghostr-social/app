mod support;

use std::collections::HashSet;
use support::fixtures::video_cache_key;
use support::native_cache::{NativeCacheHarness, VIDEO_RESPONSE};

#[tokio::test]
async fn active_valid_native_blob_survives_cache_synchronization() {
    let harness = NativeCacheHarness::new("ghostr-cache-active");
    let key = video_cache_key();
    let cached = harness.download(&key, VIDEO_RESPONSE, None).await;

    let invalid = harness
        .cache
        .retain(&HashSet::from([key]))
        .await
        .expect("retain active blob");

    assert!(invalid.is_empty());
    assert!(cached.path.exists());
    assert_eq!(*harness.used_bytes.lock().await, 5);
    std::fs::remove_dir_all(harness.directory).expect("remove cache");
}
