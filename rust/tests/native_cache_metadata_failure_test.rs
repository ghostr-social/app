mod support;

use support::fixtures::{trusted_media_client, video_cache_key};
use support::http::unused_loopback_url;
use support::native_cache::{NativeCacheHarness, VIDEO_RESPONSE};

#[tokio::test]
async fn native_blob_metadata_failure_is_reported_without_network_fallback() {
    let harness = NativeCacheHarness::new("ghostr-cache-metadata-failure");
    let key = video_cache_key();
    harness.download(&key, VIDEO_RESPONSE, None).await;
    std::fs::remove_dir_all(&harness.directory).expect("remove cache directory");
    std::fs::write(&harness.directory, []).expect("replace cache root with file");
    let unavailable = unused_loopback_url().await;

    let result = harness
        .cache
        .download(&trusted_media_client(), &key, &unavailable, None)
        .await;
    let error = match result {
        Ok(_) => panic!("metadata inspection must fail"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("inspect active native blob"));
    std::fs::remove_file(harness.directory).expect("remove blocking file");
}
