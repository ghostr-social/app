use crate::concurrency::{AdaptiveConcurrency, ConcurrencyEvidence, NetworkSetback};
use std::time::Duration;

#[test]
fn a_network_failure_reduces_an_accepted_parallel_limit() {
    let mut policy = AdaptiveConcurrency::new(2, 4);

    policy.observe(ConcurrencyEvidence {
        aggregate_bytes_per_second: 0,
        active_transfers: 2,
        saturated: false,
        ttfb: Duration::ZERO,
        setback: NetworkSetback::Failure,
    });

    assert_eq!(policy.limit(), 1);
    assert_eq!(policy.accepted_limit(), 1);
}
