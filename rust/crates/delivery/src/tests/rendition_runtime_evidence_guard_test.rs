use crate::delivery_events::{DeliveryCandidate, DeliveryPlayback};
use crate::manager::quality::select_rendition;
use crate::manager::state::DeliveryState;
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::video_rendition::VideoRendition;
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};
use std::time::Duration;

#[test]
fn runtime_quality_waits_for_both_playback_and_measured_network_evidence() {
    let post = PostId::new("adaptive");
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    let high = rendition("high", 6_000_000);
    state.apply_candidate(DeliveryCandidate {
        post: post.clone(),
        meta: high.meta().clone(),
        renditions: vec![high, rendition("low", 1_000_000)],
        discovered_at: 1,
    });
    let empty = HostStats::new();

    assert!(select_rendition(&mut state, &empty, 1_000).is_none());
    assert!(state.apply_playback(playback(post)));
    assert!(select_rendition(&mut state, &empty, 1_000).is_none());
}

fn playback(post: PostId) -> DeliveryPlayback {
    DeliveryPlayback {
        session: PlaybackSession::new(post, 1),
        sequence: PlaybackObservationSequence::new(1),
        observation: PlaybackObservation::try_new(
            Duration::ZERO,
            Duration::ZERO,
            1_000,
            PlaybackPhase::Starting,
        )
        .unwrap(),
    }
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
    .unwrap()
}
