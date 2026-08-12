use crate::adaptive::AdaptivePlayabilityPolicy;
use crate::tests::adaptive_support::snapshot;

#[test]
fn every_origin_allocation_carries_positive_gain_cost_utility_and_a_reason() {
    let plan = AdaptivePlayabilityPolicy.plan(&snapshot(8, 20_000_000, 20_000, 2));

    assert!(!plan.allocations.is_empty());
    for allocation in plan.allocations {
        assert!(allocation.expected_playable_gain_ms > 0);
        assert!(allocation.utility.expected_delivery_ms > 0);
        assert!(allocation.utility.score.is_finite() && allocation.utility.score > 0.0);
        assert!(allocation.commitment_until_ms > 10_000);
    }
}
