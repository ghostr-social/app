mod delivery_fixture;

use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_delivery::cache_registry::{CacheStatus, CacheVideo};
use ghostr_engine::{DeliveryKind, VideoMeta};

#[tokio::test]
async fn clearing_delivery_cancels_work_and_removes_every_cached_range() {
    let harness = start_harness("ghostr-delivery-clear", DeliveryOptions::default());
    harness.cache.replace([CacheVideo {
        id: "clear-me".to_owned(),
        meta: VideoMeta {
            urls: vec!["https://media.example/clear.mp4".to_owned()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(16),
            duration_ms: Some(1_000),
        },
        status: CacheStatus::Ready,
    }]);
    harness
        .store
        .write_range("clear-me", 0, &[7; 16])
        .await
        .expect("stored range");

    harness.handle.clear().await.expect("clear delivery");

    assert!(!harness.cache.contains("clear-me"));
    assert_eq!(harness.store.used_bytes().await, 0);
    assert!(harness
        .store
        .present_ranges("clear-me")
        .await
        .expect("ranges")
        .is_empty());
}
