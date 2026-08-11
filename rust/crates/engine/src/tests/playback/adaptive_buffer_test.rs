use crate::playback::{
    AdaptiveBufferPolicy, EstimateConfidence, MediaConsumption, NetworkConditions,
};
use std::time::Duration;

fn network(
    bytes_per_second: u64,
    variability: u64,
    ttfb_ms: u64,
    confidence: EstimateConfidence,
) -> NetworkConditions {
    NetworkConditions::new(
        bytes_per_second,
        variability,
        Duration::from_millis(ttfb_ms),
        confidence,
    )
}

#[test]
fn a_fast_stable_host_keeps_startup_shorter_than_steady_reserve() {
    let policy = AdaptiveBufferPolicy::default();
    let media = MediaConsumption::new(4_000_000, 1_000);

    let target = policy.target(
        network(4_000_000, 50_000, 100, EstimateConfidence::High),
        media,
    );

    assert!(target.startup() < target.steady());
    assert!(target.steady() <= Duration::from_secs(8));
}

#[test]
fn latency_and_variability_buy_more_reserve_for_the_same_media() {
    let policy = AdaptiveBufferPolicy::default();
    let media = MediaConsumption::new(4_000_000, 1_000);
    let stable = network(2_000_000, 25_000, 100, EstimateConfidence::High);
    let risky = network(2_000_000, 700_000, 1_500, EstimateConfidence::Low);

    let stable_target = policy.target(stable, media);
    let risky_target = policy.target(risky, media);

    assert!(risky_target.startup() > stable_target.startup());
    assert!(risky_target.startup() <= Duration::from_secs(6));
    assert!(risky_target.steady() > stable_target.steady());
    assert!(risky_target.emergency_horizon() > stable_target.emergency_horizon());
}

#[test]
fn a_host_that_cannot_sustain_playback_uses_the_bounded_maximum() {
    let policy = AdaptiveBufferPolicy::default();
    let media = MediaConsumption::new(8_000_000, 1_000);
    let constrained = network(700_000, 100_000, 600, EstimateConfidence::High);

    let target = policy.target(constrained, media);

    assert_eq!(target.steady(), policy.maximum());
}
