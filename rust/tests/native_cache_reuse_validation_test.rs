mod support;

use support::fixtures::{trusted_media_client, video_cache_key};
use support::http::unused_loopback_url;
use support::native_cache::{NativeCacheHarness, VIDEO_RESPONSE};

#[tokio::test]
async fn valid_native_blob_is_reused_without_contacting_the_origin() {
    let harness = NativeCacheHarness::new("ghostr-cache-reuse");
    let key = video_cache_key();
    let first = harness.download(&key, VIDEO_RESPONSE, None).await;
    let unavailable = unused_loopback_url().await;

    let second = harness
        .cache
        .download(&trusted_media_client(), &key, &unavailable, None)
        .await
        .expect("reuse cached blob");

    assert_eq!(second.path, first.path);
    assert_eq!(tokio::fs::read(&second.path).await.expect("blob"), b"video");
    assert_eq!(*harness.used_bytes.lock().await, 5);
    std::fs::remove_dir_all(harness.directory).expect("remove cache");
}
