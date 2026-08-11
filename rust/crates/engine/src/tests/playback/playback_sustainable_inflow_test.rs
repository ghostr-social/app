use crate::playback::{MediaConsumption, PlaybackObservation, PlaybackPhase};
use std::time::Duration;

#[test]
fn matching_inflow_preserves_a_healthy_playing_buffer() {
    let observation = PlaybackObservation::try_new(
        Duration::from_secs(10),
        Duration::from_secs(20),
        1_000,
        PlaybackPhase::Playing,
    )
    .expect("valid playback observation");
    let media = MediaConsumption::new(4_000_000, 1_000);

    assert!(observation.time_to_empty(4_000_000, media).is_none());
    assert!(!observation.needs_urgent_refill(4_000_000, media, Duration::from_secs(4),));
}
