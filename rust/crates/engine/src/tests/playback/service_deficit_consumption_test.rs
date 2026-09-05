use crate::playback::{BufferScenario, PlaybackPhase, UsableArrival};

#[test]
fn consumption_follows_intent_and_never_rewards_a_transport_stall() {
    let playing = BufferScenario::new(20_000, 2_000, PlaybackPhase::Playing);
    let stalled = BufferScenario::new(20_000, 2_000, PlaybackPhase::NetworkStalled);
    let paused = BufferScenario::new(20_000, 2_000, PlaybackPhase::Paused);
    assert_eq!(playing.required_ms(&[]), Ok(40_000));
    assert_eq!(stalled.required_ms(&[]), Ok(40_000));
    assert_eq!(paused.required_ms(&[]), Ok(0));
}

#[test]
fn fractional_milliseconds_round_up_and_invalid_arrivals_are_rejected() {
    let scenario = BufferScenario::new(1, 1_500, PlaybackPhase::Playing);
    assert_eq!(scenario.required_ms(&[]), Ok(2));
    let reversed = [UsableArrival::new(1, 1), UsableArrival::new(0, 2)];
    assert!(scenario.required_ms(&reversed).is_err());
    let regressed = [UsableArrival::new(0, 1), UsableArrival::new(1, 0)];
    assert!(scenario.required_ms(&regressed).is_err());
}
