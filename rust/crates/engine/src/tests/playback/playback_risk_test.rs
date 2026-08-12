use crate::playback::{MediaConsumption, PlaybackObservation, PlaybackPhase};
use std::time::Duration;

fn observation(buffer_seconds: u64, phase: PlaybackPhase) -> PlaybackObservation {
    PlaybackObservation::try_new(
        Duration::from_secs(10),
        Duration::from_secs(10 + buffer_seconds),
        1_000,
        phase,
    )
    .expect("valid observation")
}

#[test]
fn playing_becomes_urgent_only_when_the_buffer_will_empty_inside_the_horizon() {
    let media = MediaConsumption::new(4_000_000, 1_000);
    let horizon = Duration::from_secs(4);

    assert!(observation(2, PlaybackPhase::Playing).needs_urgent_refill(2_000_000, media, horizon,));
    assert!(
        !observation(10, PlaybackPhase::Playing).needs_urgent_refill(2_000_000, media, horizon,)
    );
}

#[test]
fn startup_and_a_real_network_stall_are_immediately_urgent() {
    let media = MediaConsumption::new(4_000_000, 1_000);

    for phase in [PlaybackPhase::Starting, PlaybackPhase::NetworkStalled] {
        assert!(observation(10, phase).needs_urgent_refill(
            8_000_000,
            media,
            Duration::from_secs(1),
        ));
    }
}

#[test]
fn pause_end_and_inactivity_never_report_a_network_emergency() {
    let media = MediaConsumption::new(4_000_000, 1_000);

    for phase in [
        PlaybackPhase::Paused,
        PlaybackPhase::Ended,
        PlaybackPhase::Inactive,
    ] {
        assert!(!observation(0, phase).needs_urgent_refill(0, media, Duration::from_secs(30),));
    }
}
