use crate::adaptive::AdaptivePlayabilityPolicy;
use crate::tests::adaptive_support::snapshot;

#[test]
fn new_work_keeps_at_least_the_snapshot_commitment_horizon() {
    let mut input = snapshot(2, 20_000_000, 20_000, 2);
    input.commitment_ms = 7_500;

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(!plan.allocations.is_empty());
    assert!(plan
        .allocations
        .iter()
        .all(|work| work.commitment_until_ms >= 17_500));
}

#[test]
fn commitment_hysteresis_starts_after_shared_delivery_is_expected() {
    let mut input = snapshot(2, 1_000_000, 20_000, 2);
    input.commitment_ms = 1_000;
    input.network.connection_ceiling = 3;

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(!plan.allocations.is_empty());
    assert!(plan.allocations.iter().all(|work| {
        work.commitment_until_ms
            == input.observed_at_ms + input.commitment_ms + work.utility.expected_delivery_ms * 3
    }));
}

#[test]
fn shared_slow_delivery_extends_commitment_through_expected_completion() {
    let mut input = snapshot(2, 1_000_000, 20_000, 2);
    input.commitment_ms = 1_000;
    input.network.connection_ceiling = 3;

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert!(!plan.allocations.is_empty());
    assert!(plan.allocations.iter().all(|work| {
        work.commitment_until_ms >= input.observed_at_ms + work.utility.expected_delivery_ms * 3
    }));
}
