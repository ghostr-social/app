use ghostr_delivery::cache_registry::{CacheRegistry, CacheStatus, CacheVideo};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[test]
fn route_authority_distinguishes_compatibility_and_exact_cache_entries() {
    let registry = CacheRegistry::new();
    let stored_meta = meta("https://cdn.example/a.mp4");
    let mut catalog = Catalog::new();
    let binding = catalog.upsert(PostId::new("clip"), stored_meta.clone());

    assert!(!registry.allows_binding("clip", &binding));
    registry.insert("clip");
    assert!(registry.allows_binding("clip", &binding));
    registry.replace([video(stored_meta)]);
    assert!(registry.allows_binding("clip", &binding));
    registry.replace([video(meta("https://cdn.example/b.mp4"))]);
    assert!(!registry.allows_binding("clip", &binding));
}

fn video(meta: VideoMeta) -> CacheVideo {
    CacheVideo {
        id: "clip".to_owned(),
        meta,
        status: CacheStatus::Ready,
    }
}

fn meta(source: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![source.to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(4),
        duration_ms: Some(1_000),
    }
}
