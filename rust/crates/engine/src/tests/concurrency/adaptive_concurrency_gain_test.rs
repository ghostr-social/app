use crate::concurrency::{
    AdaptiveConcurrency, ConcurrencyEvidence, ConcurrencyOccupancy, NetworkSetback,
};
use std::time::Duration;

#[test]
fn a_trial_is_kept_only_when_parallelism_improves_aggregate_throughput() {
    let mut policy = AdaptiveConcurrency::new(1, 3);

    drive_until_limit(&mut policy, 2, evidence(1, 1_000_000, 100));
    drive_until_accepted(&mut policy, 2, evidence(2, 1_300_000, 110));

    assert_eq!(policy.limit(), 2);
    assert_eq!(policy.accepted_limit(), 2);
}

fn drive_until_limit(
    policy: &mut AdaptiveConcurrency,
    expected: usize,
    evidence: ConcurrencyEvidence,
) {
    for _ in 0..20 {
        policy.observe(evidence);
        if policy.limit() == expected {
            return;
        }
    }
    panic!("concurrency did not reach {expected}");
}

fn drive_until_accepted(
    policy: &mut AdaptiveConcurrency,
    expected: usize,
    evidence: ConcurrencyEvidence,
) {
    for _ in 0..20 {
        policy.observe(evidence);
        if policy.accepted_limit() == expected {
            return;
        }
    }
    panic!("concurrency was not accepted at {expected}");
}

fn evidence(active: usize, throughput: u64, ttfb_ms: u64) -> ConcurrencyEvidence {
    ConcurrencyEvidence {
        aggregate_bytes_per_second: throughput,
        occupancy: ConcurrencyOccupancy::new(active, active),
        saturated: true,
        ttfb: Duration::from_millis(ttfb_ms),
        setback: NetworkSetback::None,
    }
}
