mod support;

use support::fixtures::video_cache_key;
use support::native_cache::{NativeCacheHarness, VIDEO_RESPONSE};

#[tokio::test]
async fn directory_at_a_native_blob_path_is_removed_before_refetch() {
    let harness = NativeCacheHarness::new("ghostr-cache-directory-blob");
    let key = video_cache_key();
    let cached = harness.download(&key, VIDEO_RESPONSE, None).await;
    tokio::fs::remove_file(&cached.path)
        .await
        .expect("remove cached blob");
    tokio::fs::create_dir(&cached.path)
        .await
        .expect("replace blob with directory");

    let repaired = harness.download(&key, VIDEO_RESPONSE, None).await;

    assert!(repaired.path.is_file());
    assert_eq!(
        tokio::fs::read(&repaired.path).await.expect("blob"),
        b"video"
    );
    assert_eq!(*harness.used_bytes.lock().await, 5);
    std::fs::remove_dir_all(harness.directory).expect("remove cache");
}
