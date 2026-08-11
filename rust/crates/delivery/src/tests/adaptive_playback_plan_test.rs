use crate::delivery_events::{DeliveryFocus, DeliveryPlayback, FocusItem};
use crate::manager::plan::playback::playback_plan;
use crate::manager::state::DeliveryState;
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};
use std::collections::HashMap;
use std::time::Duration;

#[test]
fn playback_authorizes_a_bounded_frontier_and_pause_does_not_extend_it() {
    let mut state = state();
    assert!(state.apply_playback(update(1, PlaybackPhase::Playing)));
    let urls = HashMap::from([(PostId::new("current"), media_url())]);
    let playing = playback_plan(&mut state, &HostStats::new(), &urls, 1_000, None);
    let end = playing.tail_end(&PostId::new("current")).unwrap();

    assert!(end > 0 && end < 80_000_000);
    assert!(state.apply_playback(update(2, PlaybackPhase::Paused)));
    let paused = playback_plan(&mut state, &HostStats::new(), &urls, 2_000, None);

    assert_eq!(paused.tail_end(&PostId::new("current")), Some(end));
    assert!(!paused.emergency());
}

fn state() -> DeliveryState {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    state.apply_focus(DeliveryFocus::compatibility(
        vec![FocusItem {
            post: PostId::new("current"),
            meta: VideoMeta {
                urls: vec![media_url()],
                delivery: DeliveryKind::Progressive,
                sha256: None,
                size_bytes: Some(80_000_000),
                duration_ms: Some(80_000),
            },
        }],
        0,
        0,
    ));
    state
}

fn update(sequence: u64, phase: PlaybackPhase) -> DeliveryPlayback {
    DeliveryPlayback {
        session: PlaybackSession::new(PostId::new("current"), 1),
        sequence: PlaybackObservationSequence::new(sequence),
        observation: PlaybackObservation::try_new(
            Duration::from_secs(2),
            Duration::from_secs(6),
            1_000,
            phase,
        )
        .unwrap(),
    }
}

fn media_url() -> String {
    "https://video.example/current.mp4".into()
}
