use crate::delivery_events::{DeliveryFocus, DeliveryPlayback, FocusItem, PlaybackPresentation};
use crate::manager::state::{DeliveryState, PresentationAdmission};
use ghostr_engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};
use core::time::Duration;

#[test]
fn presentation_acceptance_requires_the_current_session_and_newer_sequence() {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    let current = PlaybackSession::new(PostId::new("current"), 2);
    let early = event(current.clone(), 3);
    assert_eq!(
        state.apply_presentation(&early),
        PresentationAdmission::Pending
    );
    state.apply_focus(focus(), 0);
    assert!(state
        .apply_playback(&playback(current.clone()))
        .is_accepted());
    assert_eq!(state.take_pending_presentation(), Some(early));

    assert_eq!(
        state.apply_presentation(&event(current.clone(), 4)),
        PresentationAdmission::Accepted
    );
    assert_eq!(
        state.apply_presentation(&event(current, 4)),
        PresentationAdmission::Stale
    );
    assert_eq!(
        state.apply_presentation(&event(PlaybackSession::new(PostId::new("current"), 1), 5)),
        PresentationAdmission::Stale,
    );
}

fn event(session: PlaybackSession, sequence: u64) -> PlaybackPresentation {
    PlaybackPresentation::try_new(session, sequence, 100).expect("valid test fixture")
}

fn playback(session: PlaybackSession) -> DeliveryPlayback {
    DeliveryPlayback {
        session,
        sequence: PlaybackObservationSequence::new(1),
        observation: PlaybackObservation::try_new(
            Duration::ZERO,
            Duration::from_secs(1),
            1_000,
            PlaybackPhase::Starting,
        )
        .expect("valid test fixture"),
    }
}

fn focus() -> DeliveryFocus {
    DeliveryFocus::compatibility(
        vec![FocusItem {
            post: PostId::new("current"),
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
