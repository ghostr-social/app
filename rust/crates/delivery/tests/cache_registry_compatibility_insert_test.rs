use ghostr_delivery::cache_registry::{CacheRegistry, CacheStatus, CacheVideo};
use ghostr_engine::{DeliveryKind, VideoMeta};

#[test]
fn compatibility_registration_never_erases_exact_cache_metadata() {
    let registry = CacheRegistry::new();
    let exact = CacheVideo {
        id: "clip".to_owned(),
        meta: VideoMeta {
            urls: vec!["https://origin.example/clip.mp4".to_owned()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(16),
            duration_ms: Some(2_000),
        },
        status: CacheStatus::Complete,
    };
    registry.replace([exact.clone()]);

    registry.insert("clip");

    assert_eq!(registry.videos(), vec![exact]);
}
