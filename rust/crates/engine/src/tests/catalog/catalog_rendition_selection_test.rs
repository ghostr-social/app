use crate::adaptive::TransformKind;
use crate::catalog::Catalog;
use crate::playback::{
    AdaptiveBufferPolicy, EstimateConfidence, MediaConsumption, NetworkConditions,
    PlaybackObservation, PlaybackPhase,
};
use crate::video_rendition::VideoRendition;
use crate::{DeliveryKind, PostId, VideoMeta};
use core::time::Duration;

#[test]
fn catalog_switches_to_the_safe_representation_without_merging_mirrors() {
    let post = PostId::new("adaptive");
    let high = rendition("high", "high-mirror", 6_000_000);
    let low = rendition("low", "low-mirror", 1_000_000);
    let advertised = high.meta().clone();
    let mut catalog = Catalog::new();
    let first = catalog.upsert_with_renditions(
        post.clone(),
        high.meta().clone(),
        vec![high.clone(), low.clone()],
    );
    let unchanged =
        catalog.upsert_with_renditions(post.clone(), high.meta().clone(), vec![high, low.clone()]);
    assert_eq!(unchanged, first);
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
    .expect("fixture");
    let target =
        AdaptiveBufferPolicy::default().target(network, MediaConsumption::new(6_000_000, 1_000));

    let switched = catalog
        .select_rendition(&post, network, observation, target)
        .expect("quality changed");

    assert_ne!(switched, first);
    assert_eq!(switched.source_representation(), first.representation());
    assert!(switched.matches_source_meta(&advertised));
    assert!(!switched.matches_or_derives_from(&advertised));
    let transformed = switched
        .derive_transform(TransformKind::Remux, &"b".repeat(64))
        .expect("fixture");
    assert!(transformed.derives_from(&switched));
    assert_eq!(transformed.source_representation(), first.representation());
    assert!(transformed.matches_source_meta(&advertised));
    assert_eq!(
        catalog.lookup(&post).expect("fixture").meta,
        low.meta().clone()
    );
    assert_eq!(
        catalog.estimated_bitrate(&post, &Default::default()),
        1_000_000
    );
    assert!(catalog
        .transfer_identity(&post, "https://low.example/video.mp4")
        .is_some());
    assert!(catalog
        .transfer_identity(&post, "https://low-mirror.example/video.mp4")
        .is_some());
    assert!(catalog
        .transfer_identity(&post, "https://high.example/video.mp4")
        .is_none());
    let identity = switched
        .transfer("https://low.example/video.mp4")
        .expect("fixture");
    assert!(catalog
        .quarantine_source(&identity, "low-digest", 1)
        .contains(&post));
    assert!(catalog
        .select_rendition(&post, network, observation, target)
        .is_none());
}

fn rendition(name: &str, mirror: &str, bitrate: u64) -> VideoRendition {
    VideoRendition::try_new(
        VideoMeta {
            urls: vec![
                format!("https://{name}.example/video.mp4"),
                format!("https://{mirror}.example/video.mp4"),
            ],
            delivery: DeliveryKind::Progressive,
            sha256: Some(format!("{name}-digest")),
            size_bytes: Some(bitrate / 8),
            duration_ms: Some(1_000),
        },
        Some(bitrate),
    )
    .expect("fixture")
}
