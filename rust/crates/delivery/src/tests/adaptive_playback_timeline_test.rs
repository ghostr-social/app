use crate::delivery_events::{DeliveryFocus, DeliveryPlayback, FocusItem};
use crate::manager::plan::playback::{playback_plan, PlaybackPlanInputs};
use crate::manager::state::DeliveryState;
use crate::tests::media_timeline_fixture::classic_moov;
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::media_timeline::{parse_mp4_segments, MediaSegment};
use ghostr_engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};
use std::collections::HashMap;
use std::time::Duration;

#[test]
fn parsed_timing_authorizes_media_time_and_pause_keeps_that_horizon() {
    let mut state = state();
    let post = PostId::new("current");
    let binding = state.catalog().binding(&post).unwrap();
    let moov = classic_moov(32, 100);
    let timeline = parse_mp4_segments(&[MediaSegment::new(10_000, &moov)]).unwrap();
    assert!(state.catalog_mut().learn_timeline_for(&binding, timeline));
    assert!(state.apply_playback(update(1, PlaybackPhase::Playing)));
    let urls = HashMap::from([(post.clone(), media_url())]);

    let playing = playback_plan(
        &mut state,
        PlaybackPlanInputs {
            stats: &HostStats::new(),
            urls: &urls,
            observed_at_ms: 1_000,
            demanded_end: None,
        },
    );
    let window = playing.media_window(&post).expect("time horizon");

    assert_eq!(playing.tail_end(&post), None);
    assert!(state.apply_playback(update(2, PlaybackPhase::Paused)));
    let paused = playback_plan(
        &mut state,
        PlaybackPlanInputs {
            stats: &HostStats::new(),
            urls: &urls,
            observed_at_ms: 2_000,
            demanded_end: None,
        },
    );
    assert_eq!(paused.media_window(&post), Some(window));
    assert_eq!(paused.tail_end(&post), None);
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
