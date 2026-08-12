use ghostr_delivery::cache_registry::{CacheRegistry, CacheStatus, CacheVideo};
use ghostr_engine::{DeliveryKind, VideoMeta};

#[test]
fn cache_registry_preserves_the_delivery_projection_order() {
    let registry = CacheRegistry::new();
    registry.replace([video("z-current"), video("a-next"), video("m-later")]);

    let ids: Vec<_> = registry
        .videos()
        .into_iter()
        .map(|video| video.id)
        .collect();

    assert_eq!(ids, ["z-current", "a-next", "m-later"]);
}

fn video(id: &str) -> CacheVideo {
    CacheVideo {
        id: id.to_owned(),
        meta: VideoMeta {
            urls: vec![format!("https://media.example/{id}.mp4")],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(16),
            duration_ms: Some(1_000),
        },
        status: CacheStatus::Ready,
    }
}
