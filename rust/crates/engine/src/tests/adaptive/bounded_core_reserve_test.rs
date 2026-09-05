use crate::adaptive::AdaptivePlayabilityPolicy;
use crate::tests::adaptive_support::snapshot;

#[test]
fn rapid_navigation_cannot_expand_the_initial_encoded_reserve_beyond_two_items() {
    let input = snapshot(8, 2_000_000, 20_000, 60);
    let plan = AdaptivePlayabilityPolicy.plan(&input);
    assert!(plan.ready_reserve.target <= 2);
    assert!(plan.ready_reserve.candidates.len() <= 2);
}
