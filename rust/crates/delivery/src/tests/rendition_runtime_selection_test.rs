use crate::delivery_events::{DeliveryCandidate, DeliveryPlayback};
use crate::manager::quality::select_rendition;
use crate::manager::state::DeliveryState;
use core::time::Duration;
use ghostr_engine::host_stats::{HostStats, ThroughputSample};
use ghostr_engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::video_rendition::VideoRendition;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};

#[test]
fn measured_throughput_and_buffer_risk_switch_the_playing_catalog_representation() {
    let post = PostId::new("adaptive");
    let high = rendition("high", 6_000_000);
    let low = rendition("low", 1_000_000);
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    state.apply_candidate(DeliveryCandidate {
        post: post.clone(),
        meta: high.meta().clone(),
        preview: None,
        metadata_evidence: Vec::new(),
        renditions: vec![high, low],
        discovered_at: 1,
    });
    state.take_representation_bindings();
    assert!(state.apply_playback(&playback(post.clone())).is_accepted());
    let stats = slow_stats();

    let binding = select_rendition(&mut state, &stats, 8_000).expect("quality switch");

    assert_eq!(binding.post(), &post);
    assert_eq!(
        state
            .catalog()
            .lookup(&post)
            .expect("valid test fixture")
            .meta
            .urls,
        ["https://low.example/video.mp4"]
    );
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
        .expect("valid test fixture"),
    }
}

fn slow_stats() -> HostStats {
    let mut stats = HostStats::new();
    for second in 1..=8 {
        let sample = ThroughputSample::new(200_000, Duration::from_secs(1), second * 1_000, 1)
            .expect("valid test fixture");
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
            size_bytes: Some(bitrate / 8),
            duration_ms: Some(1_000),
        },
        Some(bitrate),
    )
    .expect("valid test fixture")
}
