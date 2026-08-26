
use crate::delivery_events::{command_channel, DeliveryCommand, DeliveryPlayback, PlaybackPresentation, PlaybackPresentationIngress};
use ghostr_engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::PostId;
use core::time::Duration;

#[test]
fn presentation_survives_latest_phase_coalescing_in_its_own_bounded_mailbox() {
    let (handle, mut receiver) = command_channel();
    let session = PlaybackSession::new(PostId::new("video"), 4);
    handle.report_playback(playback(session.clone(), 1, PlaybackPhase::Playing));
    assert_eq!(
        handle.report_playback_presentation(presentation(session.clone(), 9)),
        PlaybackPresentationIngress::Accepted,
    );
    handle.report_playback(playback(session.clone(), 2, PlaybackPhase::Inactive));

    let DeliveryCommand::Playback(latest) = receiver.try_control().expect("valid test fixture") else {
        panic!("expected playback");
    };
    assert_eq!(latest.sequence, PlaybackObservationSequence::new(2));
    assert_eq!(
        receiver.try_playback_presentation(),
        Some(presentation(session, 9))
    );
    for sequence in 10..18 {
        assert_eq!(
            handle.report_playback_presentation(presentation(
                PlaybackSession::new(PostId::new("video"), sequence),
                sequence,
            )),
            PlaybackPresentationIngress::Accepted,
        );
    }
    assert_eq!(
        handle.report_playback_presentation(presentation(
            PlaybackSession::new(PostId::new("video"), 18),
            18,
        )),
        PlaybackPresentationIngress::Saturated,
    );
}

fn presentation(session: PlaybackSession, sequence: u64) -> PlaybackPresentation {
    PlaybackPresentation::try_new(session, sequence, 450).expect("valid test fixture")
}

fn playback(session: PlaybackSession, sequence: u64, phase: PlaybackPhase) -> DeliveryPlayback {
    DeliveryPlayback {
        session,
        sequence: PlaybackObservationSequence::new(sequence),
        observation: PlaybackObservation::try_new(
            Duration::ZERO,
            Duration::from_secs(2),
            1_000,
            phase,
        )
        .expect("valid test fixture"),
    }
}
