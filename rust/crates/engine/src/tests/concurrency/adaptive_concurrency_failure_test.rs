use crate::concurrency::{
    AdaptiveConcurrency, ConcurrencyEvidence, ConcurrencyOccupancy, NetworkSetback,
};
use std::time::Duration;

#[test]
fn a_network_failure_reduces_an_accepted_parallel_limit() {
    let mut policy = AdaptiveConcurrency::new(2, 4);

    policy.observe(ConcurrencyEvidence {
        aggregate_bytes_per_second: 0,
        occupancy: ConcurrencyOccupancy::new(2, 2),
        saturated: false,
        ttfb: Duration::ZERO,
        setback: NetworkSetback::Failure,
    });

    assert_eq!(policy.limit(), 1);
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
