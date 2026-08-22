use crate::delivery_events::{DeliveryFocus, DeliveryPlayback, FocusItem, PlaybackPresentation};
use crate::manager::state::{DeliveryState, PresentationAdmission};
use ghostr_engine::evidence::{EvidenceField, EvidenceValue};
use ghostr_engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::{DataUsageLevel, DeliveryKind, EngineParams, PostId, VideoMeta};
use std::time::Duration;

const URL: &str = "https://media.example/video.mp4";

#[test]
fn accepted_presented_frame_and_terminal_failure_label_readiness() {
    let mut presented = state();
    let session = PlaybackSession::new(PostId::new("clip"), 1);
    assert!(presented
        .apply_playback(update(session.clone(), 1, PlaybackPhase::Starting))
        .is_accepted());
    let event = PlaybackPresentation::try_new(session, 1, 200).unwrap();
    assert_eq!(
        presented.apply_presentation(event),
        PresentationAdmission::Accepted
    );
    assert_eq!(readiness(&presented, 200), Some(EvidenceValue::Ready(true)));

    let mut failed = state();
    let session = PlaybackSession::new(PostId::new("clip"), 1);
    assert!(failed
        .apply_playback_at(update(session, 1, PlaybackPhase::Failed), 300)
        .is_accepted());
    assert_eq!(readiness(&failed, 300), Some(EvidenceValue::Ready(false)));
}

fn state() -> DeliveryState {
    let mut state = DeliveryState::new(EngineParams::default(), DataUsageLevel::Balanced);
    state.apply_focus(
        DeliveryFocus::compatibility(
            vec![FocusItem {
                post: PostId::new("clip"),
                meta: VideoMeta {
                    urls: vec![URL.into()],
                    delivery: DeliveryKind::Progressive,
                    sha256: None,
                    size_bytes: Some(16),
                    duration_ms: Some(1_000),
                },
            }],
            0,
            0,
        ),
        0,
    );
    state
}

fn update(session: PlaybackSession, sequence: u64, phase: PlaybackPhase) -> DeliveryPlayback {
    DeliveryPlayback {
        session,
        sequence: PlaybackObservationSequence::new(sequence),
        observation: PlaybackObservation::try_new(
            Duration::ZERO,
            Duration::from_secs(1),
            1_000,
            phase,
        )
        .unwrap(),
    }
}

fn readiness(state: &DeliveryState, now_ms: u64) -> Option<EvidenceValue> {
    state
        .catalog()
        .lookup(&PostId::new("clip"))?
        .evidence_assessment_for(URL, now_ms)
        .value(EvidenceField::Readiness)
        .cloned()
}
