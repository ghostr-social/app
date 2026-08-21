use crate::concurrency::{
    AdaptiveConcurrency, ConcurrencyEvidence, ConcurrencyOccupancy, NetworkSetback,
};
use std::time::Duration;

#[test]
fn a_filled_trial_does_not_require_demand_for_another_slot() {
    let mut policy = AdaptiveConcurrency::new(1, 3);

    for _ in 0..4 {
        assert_eq!(policy.observe(sample(1, 1_000_000, false)), 1);
    }
    for _ in 0..4 {
        policy.observe(sample(1, 1_000_000, true));
    }
    for _ in 0..4 {
        assert_eq!(policy.observe(sample(3, 1_300_000, false)), 2);
    }

    assert_eq!(policy.observe(sample(2, 1_300_000, true)), 3);
}

fn sample(active: usize, throughput: u64, saturated: bool) -> ConcurrencyEvidence {
    ConcurrencyEvidence {
        aggregate_bytes_per_second: throughput,
        occupancy: ConcurrencyOccupancy::new(active, active),
        saturated,
        ttfb: Duration::from_millis(100),
        setback: NetworkSetback::None,
    }
}
