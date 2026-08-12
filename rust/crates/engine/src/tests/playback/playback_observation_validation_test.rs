use crate::playback::{PlaybackObservation, PlaybackObservationError, PlaybackPhase};
use std::time::Duration;

#[test]
fn an_observation_preserves_the_measured_playback_frontier_and_rate() {
    let observation = PlaybackObservation::try_new(
        Duration::from_secs(7),
        Duration::from_secs(12),
        1_250,
        PlaybackPhase::Playing,
    )
    .expect("valid playback observation");

    assert_eq!(observation.position(), Duration::from_secs(7));
    assert_eq!(observation.buffer_ahead(), Duration::from_secs(5));
    assert_eq!(observation.playback_rate_milli(), 1_250);
    assert_eq!(observation.phase(), PlaybackPhase::Playing);
}

#[test]
fn invalid_frontiers_and_rates_are_rejected_at_the_boundary() {
    let invalid_frontier = PlaybackObservation::try_new(
        Duration::from_secs(2),
        Duration::from_secs(1),
        1_000,
        PlaybackPhase::Playing,
    );
    let invalid_rate =
        PlaybackObservation::try_new(Duration::ZERO, Duration::ZERO, 0, PlaybackPhase::Playing);

    assert_eq!(
        invalid_frontier,
        Err(PlaybackObservationError::BufferedBeforePosition),
    );
    assert_eq!(
        invalid_rate,
        Err(PlaybackObservationError::ZeroPlaybackRate),
    );
}
