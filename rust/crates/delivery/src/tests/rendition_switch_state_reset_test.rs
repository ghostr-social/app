use crate::delivery_events::{DeliveryCandidate, DeliveryPlayback};
use crate::manager::quality::{prepare_rendition_switch, select_rendition};
use crate::manager::retry::{RetryBook, RetryPolicy};
use crate::manager::state::DeliveryState;
use crate::probe::pool::MetadataProbePool;
use ghostr_engine::host_stats::{HostStats, ThroughputSample};
use ghostr_engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::video_rendition::VideoRendition;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};
use std::time::Duration;

#[test]
fn selected_rendition_resets_representation_fenced_delivery_state() {
    let post = PostId::new("adaptive");
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    let high = rendition("high", 6_000_000);
    state.apply_candidate(DeliveryCandidate {
        post: post.clone(),
        meta: high.meta().clone(),
        preview: None,
        metadata_evidence: Vec::new(),
        renditions: vec![high, rendition("low", 1_000_000)],
        discovered_at: 1,
    });
    state.take_representation_bindings();
    assert!(state.apply_playback(playback(post.clone())).is_accepted());
    let binding = select_rendition(&mut state, &slow_stats(), 8_000).unwrap();
    let mut probes = MetadataProbePool::new(1);
    probes.learned(&post);
    let mut retry = RetryBook::new(RetryPolicy::default());
    assert!(retry.expedite_demand(&post, 8));
    assert!(retry.cool_down(post.clone()).is_none());
    assert!(retry.cool_down(post.clone()).is_some());

    prepare_rendition_switch(&mut state, &mut probes, &mut retry, binding.clone());

    assert_eq!(state.take_representation_bindings(), [binding]);
    assert!(!retry.is_cooling(&post));
    assert!(retry.expedite_demand(&post, 8));
    assert_eq!(probes.claim(state.catalog(), &[post], &retry).len(), 1);
}

fn playback(post: PostId) -> DeliveryPlayback {
    DeliveryPlayback {
        session: PlaybackSession::new(post, 1),
        sequence: PlaybackObservationSequence::new(1),
        observation: PlaybackObservation::try_new(
            Duration::ZERO,
            Duration::from_secs(1),
            1_000,
            PlaybackPhase::NetworkStalled,
        )
        .unwrap(),
    }
}

fn slow_stats() -> HostStats {
    let mut stats = HostStats::new();
    for second in 1..=8 {
        let sample =
            ThroughputSample::new(200_000, Duration::from_secs(1), second * 1_000, 1).unwrap();
        stats.record_host_throughput("high.example", sample);
    }
    stats
}

fn rendition(name: &str, bitrate: u64) -> VideoRendition {
    VideoRendition::try_new(
        VideoMeta {
            urls: vec![format!("https://{name}.example/video.mp4")],
            delivery: DeliveryKind::Progressive,
            sha256: Some(format!("{name}-digest")),
            size_bytes: None,
            duration_ms: Some(1_000),
        },
        Some(bitrate),
    )
    .unwrap()
}
