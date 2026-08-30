use super::binding_is_current;
use core::time::Duration;
use ghostr_delivery::cache_registry::{CacheRegistry, CacheStatus, CacheVideo};
use ghostr_engine::catalog::Catalog;
use ghostr_engine::playback::{
    AdaptiveBufferPolicy, EstimateConfidence, MediaConsumption, NetworkConditions,
    PlaybackObservation, PlaybackPhase,
};
use ghostr_engine::video_rendition::VideoRendition;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};

#[cfg(not(feature = "video-debug-web"))]
mod cache_wake_test;
#[cfg(not(feature = "video-debug-web"))]
mod known_extent_test;

#[test]
fn issuance_requires_the_active_selected_rendition_not_only_the_advertised_root() {
    let advertised = meta("high");
    let selected_meta = meta("low");
    let selected = selected_binding(&advertised, &selected_meta);
    let cache = CacheRegistry::new();
    cache.replace([cached(advertised.clone())]);

    assert!(!binding_is_current(&cache, "clip", &advertised, &selected));
    cache.replace([cached(selected_meta)]);
    assert!(binding_is_current(&cache, "clip", &advertised, &selected));
}

fn selected_binding(
    advertised: &VideoMeta,
    selected: &VideoMeta,
) -> ghostr_engine::representation::RepresentationBinding {
    let variants = vec![
        variant(advertised.clone(), 6_000_000),
        variant(selected.clone(), 1_000_000),
    ];
    let mut catalog = Catalog::new();
    let post = PostId::new("clip");
    catalog.upsert_with_renditions(post.clone(), advertised.clone(), variants);
    let network = NetworkConditions::new(
        250_000,
        0,
        Duration::from_millis(100),
        EstimateConfidence::High,
    );
    let observation = PlaybackObservation::try_new(
        Duration::ZERO,
        Duration::from_secs(1),
        1_000,
        PlaybackPhase::NetworkStalled,
    )
    .expect("valid test fixture");
    let target =
        AdaptiveBufferPolicy::default().target(network, MediaConsumption::new(6_000_000, 1_000));
    catalog
        .select_rendition(&post, network, observation, target)
        .expect("valid test fixture")
}

fn cached(meta: VideoMeta) -> CacheVideo {
    CacheVideo {
        id: "clip".to_owned(),
        meta,
        status: CacheStatus::Complete,
    }
}

fn variant(meta: VideoMeta, bitrate: u64) -> VideoRendition {
    VideoRendition::try_new(meta, Some(bitrate)).expect("valid test fixture")
}

fn meta(name: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![format!("https://{name}.example/video.mp4")],
        delivery: DeliveryKind::Progressive,
        sha256: Some(format!("{name}-digest")),
        size_bytes: Some(16),
        duration_ms: Some(2_000),
    }
}
