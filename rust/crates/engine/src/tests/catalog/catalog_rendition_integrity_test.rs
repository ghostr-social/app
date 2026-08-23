use crate::catalog::Catalog;
use crate::playback::{
    AdaptiveBufferPolicy, EstimateConfidence, MediaConsumption, NetworkConditions,
    PlaybackObservation, PlaybackPhase,
};
use crate::video_rendition::VideoRendition;
use crate::{DeliveryKind, PostId, VideoMeta};
use std::time::Duration;

#[test]
fn catalog_refuses_a_rendition_with_a_known_quarantined_digest() {
    let digest = "d".repeat(64);
    let mut catalog = Catalog::new();
    quarantine_digest(&mut catalog, &digest);
    let post = PostId::new("adaptive");
    let high = meta("high", &"a".repeat(64));
    let low = meta("low", &digest);
    let variants = vec![variant(high.clone(), 6_000_000), variant(low, 1_000_000)];
    catalog.upsert_with_renditions(post.clone(), high, variants);
    let (network, observation, target) = stalled_selection();
    let selected = catalog.select_rendition(&post, network, observation, target);

    assert!(selected.is_none());
    let active = catalog.lookup(&post).unwrap();
    assert_eq!(active.meta.urls, vec!["https://high.example/video.mp4"]);
    assert!(!active.is_quarantined());
}

fn quarantine_digest(catalog: &mut Catalog, digest: &str) {
    let post = PostId::new("failed");
    let binding = catalog.upsert(post, meta("failed", digest));
    let identity = binding
        .transfer("https://failed.example/video.mp4")
        .unwrap();
    catalog.quarantine_mirror_group(&identity, digest, 1);
}

fn stalled_selection() -> (
    NetworkConditions,
    PlaybackObservation,
    crate::playback::BufferTarget,
) {
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
    .unwrap();
    let target =
        AdaptiveBufferPolicy::default().target(network, MediaConsumption::new(6_000_000, 1_000));
    (network, observation, target)
}

fn variant(meta: VideoMeta, bitrate: u64) -> VideoRendition {
    VideoRendition::try_new(meta, Some(bitrate)).unwrap()
}

fn meta(name: &str, digest: &str) -> VideoMeta {
    VideoMeta {
        urls: vec![format!("https://{name}.example/video.mp4")],
        delivery: DeliveryKind::Progressive,
        sha256: Some(digest.to_owned()),
        size_bytes: Some(16),
        duration_ms: Some(2_000),
    }
}
