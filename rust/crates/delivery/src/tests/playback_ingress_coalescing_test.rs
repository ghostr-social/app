use crate::delivery_events::{command_channel, DeliveryCommand, DeliveryPlayback};
use ghostr_engine::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
};
use ghostr_engine::PostId;
use std::time::Duration;

#[test]
fn a_late_stale_sample_cannot_replace_newer_pending_playback_evidence() {
    let (handle, mut receiver) = command_channel();
    handle.report_playback(update(9));
    handle.report_playback(update(8));

    let DeliveryCommand::Playback(latest) = receiver.try_control().expect("playback") else {
        panic!("expected playback command");
    };
    assert_eq!(latest.sequence, PlaybackObservationSequence::new(9));
}

fn update(sequence: u64) -> DeliveryPlayback {
    DeliveryPlayback {
        session: PlaybackSession::new(PostId::new("video"), 4),
        sequence: PlaybackObservationSequence::new(sequence),
        observation: PlaybackObservation::try_new(
            Duration::from_secs(1),
            Duration::from_secs(5),
            1_000,
            PlaybackPhase::Playing,
        )
        .unwrap(),
    }
}
