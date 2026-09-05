use crate::adaptive::AdaptivePlayabilityPolicy;
use crate::tests::adaptive_support::{frontier, snapshot};

#[test]
fn high_round_trip_time_contracts_speculative_coverage_at_equal_throughput() {
    let policy = AdaptivePlayabilityPolicy;
    let low_rtt = snapshot(16, 20_000_000, 20_000, 2);
    let mut high_rtt = low_rtt.clone();
    high_rtt.network.rtt_ms = 2_000;

    let low_rtt = policy.plan(&low_rtt);
    let high_rtt = policy.plan(&high_rtt);

    assert!(frontier(&low_rtt).len() >= frontier(&high_rtt).len());
}
