use crate::catalog::{Catalog, CatalogEvidenceState};
use crate::playback::{
    AdaptiveBufferPolicy, EstimateConfidence, MediaConsumption, NetworkConditions,
    PlaybackObservation, PlaybackPhase,
};
use crate::video_rendition::VideoRendition;
use crate::{DeliveryKind, PostId, VideoMeta};
use std::time::Duration;

#[test]
fn live_and_pruned_posts_persist_only_their_advertised_digest_claim() {
    let high_digest = "a".repeat(64);
    let low_digest = "b".repeat(64);
    let high = meta("high", &high_digest);
    let low = meta("low", &low_digest);
    let post = PostId::new("adaptive");
    for prune_before_save in [false, true] {
        let mut before = selected_catalog(post.clone(), high.clone(), low.clone());
        if prune_before_save {
            before.retain(|_| false);
        }
        let state = CatalogEvidenceState::from_json(&before.evidence_state().to_json()).unwrap();
        let mut restarted = Catalog::new();
        restarted.replace_evidence_state(state, 1);
        restarted.upsert(post.clone(), high.clone());
        let failed = restarted.upsert(PostId::new("failed-low"), low.clone());
        let identity = failed.transfer("https://low.example/video.mp4").unwrap();
        let invalidated = restarted.quarantine_mirror_group(&identity, &low_digest, 2);

        assert!(!invalidated.contains(&post), "pruned={prune_before_save}");
        assert!(!restarted.lookup(&post).unwrap().is_quarantined());
    }
}

fn selected_catalog(post: PostId, high: VideoMeta, low: VideoMeta) -> Catalog {
    let mut catalog = Catalog::new();
    catalog.upsert_with_renditions(
        post.clone(),
        high.clone(),
        vec![variant(high, 6_000_000), variant(low, 1_000_000)],
    );
    let (network, observation, target) = stalled_selection();
    catalog
        .select_rendition(&post, network, observation, target)
        .unwrap();
    catalog
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
