use crate::delivery_events::{DeliveryFocus, DeliveryPlayback, FocusItem};
use crate::manager::plan::playback::{playback_plan, PlaybackPlanInputs};
use crate::manager::state::DeliveryState;
use ghostr_engine::host_stats::{HostStats, ThroughputSample};
use ghostr_engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};
use std::collections::HashMap;
use std::time::Duration;

#[test]
fn a_zero_byte_live_sample_requests_emergency_refill_without_invalid_rate_math() {
    let post = PostId::new("current");
    let url = "https://video.example/current.mp4".to_owned();
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    state.apply_focus(DeliveryFocus::compatibility(
        vec![FocusItem {
            post: post.clone(),
            meta: VideoMeta {
                urls: vec![url.clone()],
                delivery: DeliveryKind::Progressive,
                sha256: None,
                size_bytes: Some(80_000_000),
                duration_ms: Some(80_000),
            },
        }],
        0,
        0,
    ));
    assert!(state.apply_playback(playback(post.clone())));
    let mut stats = HostStats::new();
    stats.record_overall_throughput(
        ThroughputSample::new(0, Duration::from_secs(1), 1_000, 1).unwrap(),
    );

    let plan = playback_plan(
        &mut state,
        PlaybackPlanInputs {
            stats: &stats,
            urls: &HashMap::from([(post, url)]),
            observed_at_ms: 1_000,
            demanded_end: None,
        },
    );

    assert!(plan.emergency());
}

fn playback(post: PostId) -> DeliveryPlayback {
    DeliveryPlayback {
        session: PlaybackSession::new(post, 1),
        sequence: PlaybackObservationSequence::new(1),
        observation: PlaybackObservation::try_new(
            Duration::ZERO,
            Duration::ZERO,
            1_000,
            PlaybackPhase::NetworkStalled,
        )
        .unwrap(),
    }
}
