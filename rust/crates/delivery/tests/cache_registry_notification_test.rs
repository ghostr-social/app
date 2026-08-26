use core::time::Duration;
use ghostr_delivery::cache_registry::{CacheRegistry, CacheStatus, CacheVideo};
use ghostr_engine::{DeliveryKind, VideoMeta};

#[tokio::test]
async fn identical_replacement_does_not_publish_a_cache_change() {
    let registry = CacheRegistry::new();
    registry.replace([video(CacheStatus::Ready)]);
    let changed = registry.notifier();
    let mut notification = Box::pin(changed.notified());
    notification.as_mut().enable();

    registry.replace([video(CacheStatus::Ready)]);
    assert!(
        tokio::time::timeout(Duration::from_millis(20), &mut notification)
            .await
            .is_err()
    );

    registry.replace([video(CacheStatus::Complete)]);
    tokio::time::timeout(Duration::from_secs(1), notification)
        .await
        .expect("valid test fixture");
}

fn video(status: CacheStatus) -> CacheVideo {
    CacheVideo {
        id: "post".to_owned(),
        meta: VideoMeta {
            urls: vec!["https://cdn.example/video.mp4".to_owned()],
            delivery: DeliveryKind::Progressive,
            sha256: None,
            size_bytes: Some(16),
            duration_ms: Some(2_000),
        },
        status,
    }
}
