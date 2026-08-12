use crate::playback::{
    PlaybackObservation, PlaybackObservationSequence, PlaybackPhase, PlaybackSession,
    PlaybackStatus,
};
use crate::PostId;
use std::time::Duration;

#[test]
fn only_the_active_generation_accepts_increasing_observations() {
    let old = PlaybackSession::new(PostId::new("video-a"), 4);
    let active = PlaybackSession::new(PostId::new("video-b"), 5);
    let mut status = PlaybackStatus::default();

    assert!(status.activate(old.clone()));
    assert!(status.activate(active.clone()));
    assert!(!status.activate(old.clone()));
    assert!(!status.report(&old, sequence(1), observation()));
    assert!(status.report(&active, sequence(2), observation()));
    assert!(!status.report(&active, sequence(2), observation()));
    status.discard_session();
    assert!(status.observation().is_none());
    assert!(!status.activate(old));
}

fn sequence(value: u64) -> PlaybackObservationSequence {
    PlaybackObservationSequence::new(value)
}

fn observation() -> PlaybackObservation {
    PlaybackObservation::try_new(
        Duration::from_secs(2),
        Duration::from_secs(8),
        1_000,
        PlaybackPhase::Playing,
    )
    .unwrap()
}
