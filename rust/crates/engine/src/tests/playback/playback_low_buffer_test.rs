use crate::playback::{MediaConsumption, PlaybackObservation, PlaybackPhase};
use std::time::Duration;

#[test]
fn a_nearly_empty_playing_buffer_is_urgent_even_after_a_fast_sample() {
    let observation = PlaybackObservation::try_new(
        Duration::from_secs(10),
        Duration::from_secs(11),
        1_000,
        PlaybackPhase::Playing,
    )
    .unwrap();

    assert!(observation.needs_urgent_refill(
        20_000_000,
        MediaConsumption::new(4_000_000, 1_000),
        Duration::from_secs(4),
    ));
}
