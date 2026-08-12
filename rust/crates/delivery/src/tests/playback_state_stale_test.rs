use crate::delivery_events::{DeliveryFocus, DeliveryPlayback, FocusItem};
use crate::manager::state::DeliveryState;
use ghostr_engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};
use std::time::Duration;

#[test]
fn state_accepts_only_current_session_and_increasing_sequence() {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    state.apply_focus(focus("current"), 0);

    assert!(state.apply_playback(update("current", 2, 3)));
    assert!(!state.apply_playback(update("other", 3, 1)));
    assert!(!state.apply_playback(update("current", 2, 2)));
    assert_eq!(
        state
            .playback()
            .session()
            .map(|session| session.generation()),
        Some(2)
    );
}

#[test]
fn returning_to_a_post_does_not_revive_an_older_playback_session() {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    state.apply_focus(focus("current"), 0);
    assert!(state.apply_playback(update("current", 2, 1)));

    state.apply_focus(focus("other"), 1);
    state.apply_focus(focus("current"), 2);

    assert!(!state.apply_playback(update("current", 1, 2)));
}

fn update(post: &str, generation: u64, sequence: u64) -> DeliveryPlayback {
    DeliveryPlayback {
        session: PlaybackSession::new(PostId::new(post), generation),
        sequence: PlaybackObservationSequence::new(sequence),
        observation: PlaybackObservation::try_new(
            Duration::ZERO,
            Duration::from_secs(4),
            1_000,
            PlaybackPhase::Playing,
        )
        .unwrap(),
    }
}

fn focus(post: &str) -> DeliveryFocus {
    DeliveryFocus::compatibility(
        vec![FocusItem {
            post: PostId::new(post),
            meta: VideoMeta {
                urls: vec!["https://media.example/video.mp4".into()],
                delivery: DeliveryKind::Progressive,
                sha256: None,
                size_bytes: Some(16),
                duration_ms: Some(1_000),
            },
        }],
        0,
        0,
    )
}
