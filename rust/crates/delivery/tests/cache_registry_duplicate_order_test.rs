use ghostr_delivery::cache_registry::{CacheRegistry, CacheStatus, CacheVideo};
use ghostr_engine::{DeliveryKind, VideoMeta};

#[test]
fn duplicate_projection_entries_keep_their_first_position_and_latest_value() {
    let registry = CacheRegistry::new();
    registry.insert("pending");
    registry.insert("pending");
    assert!(registry.videos().is_empty());

    registry.replace([
        video("first", CacheStatus::Partial),
        video("second", CacheStatus::Ready),
        video("first", CacheStatus::Complete),
    ]);
    let videos = registry.videos();

    assert_eq!(
        videos
            .iter()
            .map(|video| video.id.as_str())
            .collect::<Vec<_>>(),
        ["first", "second"]
    );
    assert_eq!(videos[0].status, CacheStatus::Complete);
}

fn video(id: &str, status: CacheStatus) -> CacheVideo {
    CacheVideo {
        id: id.to_owned(),
        meta: VideoMeta {
            urls: vec![format!("https://media.example/{id}.mp4")],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(16),
            duration_ms: Some(1_000),
        },
        status,
    }
}
