use crate::concurrency::{
    AdaptiveConcurrency, ConcurrencyEvidence, ConcurrencyOccupancy, NetworkSetback,
};
use core::time::Duration;

#[test]
fn a_stall_or_failed_trial_returns_to_the_last_safe_limit() {
    let mut policy = AdaptiveConcurrency::new(1, 3);
    reach_trial(&mut policy);

    for _ in 0..20 {
        policy.observe(evidence(2, 1_020_000, 180, NetworkSetback::None));
        if policy.limit() == 1 {
            break;
        }
    }
    assert_eq!(policy.limit(), 1);

    reach_trial(&mut policy);
    policy.observe(evidence(2, 1_400_000, 100, NetworkSetback::Stall));
    assert_eq!(policy.limit(), 1);
}

fn reach_trial(policy: &mut AdaptiveConcurrency) {
    for _ in 0..40 {
        policy.observe(evidence(1, 1_000_000, 100, NetworkSetback::None));
        if policy.limit() == 2 {
            return;
        }
    }
    panic!("trial did not start");
}

fn evidence(
    active: usize,
    throughput: u64,
    ttfb_ms: u64,
    setback: NetworkSetback,
) -> ConcurrencyEvidence {
    ConcurrencyEvidence {
        aggregate_bytes_per_second: throughput,
        occupancy: ConcurrencyOccupancy::new(active, active),
        saturated: true,
        ttfb: Duration::from_millis(ttfb_ms),
        setback,
    }
}
