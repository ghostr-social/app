use crate::concurrency::{
    AdaptiveConcurrency, ConcurrencyEvidence, ConcurrencyOccupancy, NetworkSetback,
};
use core::time::Duration;

#[test]
fn four_consecutive_unclaimed_windows_abandon_a_trial() {
    let mut policy = AdaptiveConcurrency::new(1, 2);
    start_trial(&mut policy);

    for _ in 0..3 {
        assert_eq!(policy.observe(underfilled_trial()), 2);
    }
    assert_eq!(policy.observe(valid_trial()), 2);
    for _ in 0..3 {
        assert_eq!(policy.observe(underfilled_trial()), 2);
    }
    assert_eq!(policy.observe(underfilled_trial()), 1);

    for _ in 0..3 {
        assert_eq!(policy.observe(baseline()), 1);
    }
    assert_eq!(policy.observe(baseline()), 2);
}

#[test]
fn a_claimed_request_pauses_trial_abandonment_until_released() {
    let mut policy = AdaptiveConcurrency::new(1, 2);
    start_trial(&mut policy);

    for _ in 0..6 {
        assert_eq!(policy.observe(pending_trial()), 2);
    }
    for _ in 0..3 {
        assert_eq!(policy.observe(silent_unclaimed_trial()), 2);
    }
    assert_eq!(policy.observe(silent_unclaimed_trial()), 1);
}

fn start_trial(policy: &mut AdaptiveConcurrency) {
    for _ in 0..4 {
        policy.observe(baseline());
    }
    assert_eq!(policy.limit(), 2);
}

fn baseline() -> ConcurrencyEvidence {
    evidence(1_000_000, ConcurrencyOccupancy::new(1, 1), true)
}

fn valid_trial() -> ConcurrencyEvidence {
    evidence(1_300_000, ConcurrencyOccupancy::new(2, 2), false)
}

fn underfilled_trial() -> ConcurrencyEvidence {
    evidence(1_000_000, ConcurrencyOccupancy::new(1, 2), false)
}

fn silent_unclaimed_trial() -> ConcurrencyEvidence {
    evidence(0, ConcurrencyOccupancy::new(1, 2), false)
}

fn pending_trial() -> ConcurrencyEvidence {
    let occupancy = ConcurrencyOccupancy::new(1, 2).with_claimed_requests(2);
    evidence(0, occupancy, false)
}

fn evidence(
    throughput: u64,
    occupancy: ConcurrencyOccupancy,
    saturated: bool,
) -> ConcurrencyEvidence {
    ConcurrencyEvidence {
        aggregate_bytes_per_second: throughput,
        occupancy,
        saturated,
        ttfb: Duration::from_millis(100),
        setback: NetworkSetback::None,
    }
}
