use ghostr_delivery::cache_registry::{CacheRegistry, CacheStatus, CacheVideo};
use ghostr_engine::{DeliveryKind, VideoMeta};

#[test]
fn cache_registry_projects_stable_ready_partial_and_complete_entries() {
    let registry = CacheRegistry::new();
    registry.insert("pending");
    assert!(registry.contains("pending"));

    registry.replace([
        video("second", CacheStatus::Partial),
        video("first", CacheStatus::Complete),
    ]);

    let videos = registry.videos();
    assert_eq!(videos[0].id, "first");
    assert_eq!(videos[0].status, CacheStatus::Complete);
    assert_eq!(videos[1].status, CacheStatus::Partial);
}

fn video(id: &str, status: CacheStatus) -> CacheVideo {
    CacheVideo {
        id: id.to_owned(),
        meta: metadata(id),
        status,
    }
}

fn metadata(id: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![format!("https://media.example/{id}.mp4")],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(16),
        duration_ms: Some(1_000),
    }
}
