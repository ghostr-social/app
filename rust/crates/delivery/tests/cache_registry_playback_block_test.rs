use ghostr_delivery::cache_registry::{CacheRegistry, CacheStatus, CacheVideo};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[test]
fn blocked_binding_remains_observable_but_cannot_be_served() {
    let post = PostId::new("clip");
    let meta = VideoMeta {
        urls: vec!["https://cdn.example/clip.mp4".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(4),
        duration_ms: Some(1_000),
    };
    let binding = Catalog::new().upsert(post, meta.clone());
    let registry = CacheRegistry::new();
    registry.replace_with_blocked(
        [CacheVideo {
            id: "clip".to_owned(),
            meta,
            status: CacheStatus::Ready,
        }],
        ["clip".to_owned()],
    );

    assert!(registry.is_playback_blocked("clip", &binding));
    assert!(registry.video_for_binding("clip", &binding).is_some());
    assert!(!registry.matches_binding("clip", &binding));
    assert!(!registry.allows_binding("clip", &binding));
}
