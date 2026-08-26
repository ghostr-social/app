use crate::concurrency::{
    AdaptiveConcurrency, ConcurrencyEvidence, ConcurrencyOccupancy, NetworkSetback,
};
use core::time::Duration;

#[test]
fn a_network_failure_reduces_an_accepted_parallel_limit() {
    let mut policy = AdaptiveConcurrency::new(2, 4);
    assert!(policy.demand_expansion_allowed());

    policy.observe(ConcurrencyEvidence {
        aggregate_bytes_per_second: 0,
        occupancy: ConcurrencyOccupancy::new(2, 2),
        saturated: false,
        ttfb: Duration::ZERO,
        setback: NetworkSetback::Failure,
    });

    assert_eq!(policy.limit(), 1);
    assert!(!policy.demand_expansion_allowed());
}

#[test]
fn demand_expansion_returns_only_after_healthy_backoff_evidence() {
    let mut policy = AdaptiveConcurrency::new(2, 4);
    policy.observe(evidence(NetworkSetback::Failure));

    for _ in 0..7 {
        policy.observe(evidence(NetworkSetback::None));
        assert!(!policy.demand_expansion_allowed());
    }
    policy.observe(evidence(NetworkSetback::None));

    assert!(policy.demand_expansion_allowed());
}

#[test]
fn severe_packet_loss_returns_parallelism_to_one_immediately() {
    let mut policy = AdaptiveConcurrency::new(3, 4);

    policy.observe(ConcurrencyEvidence {
        aggregate_bytes_per_second: 0,
        occupancy: ConcurrencyOccupancy::new(3, 3),
        saturated: false,
        ttfb: Duration::ZERO,
        setback: NetworkSetback::SevereLoss,
    });

    assert_eq!(policy.limit(), 1);
}

fn evidence(setback: NetworkSetback) -> ConcurrencyEvidence {
    ConcurrencyEvidence {
        aggregate_bytes_per_second: 1_000_000,
        occupancy: ConcurrencyOccupancy::new(1, 1),
        saturated: true,
        ttfb: Duration::from_millis(100),
        setback,
    }
}
