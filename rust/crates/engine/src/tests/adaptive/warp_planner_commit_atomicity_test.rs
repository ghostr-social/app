use super::warp_planner_commit_test_support::{planner, selected, OBSERVED_AT_MS};
use crate::adaptive::ResourceCost;

#[test]
fn insufficient_tokens_reject_without_refill_or_partial_charge() {
    let mut planner = planner(100, 10);
    let selected = selected(ResourceCost::new(100, 0, 0, 0));

    assert!(planner.commit(&selected, ResourceCost::new(90, 0, 0, 0), OBSERVED_AT_MS,));
    assert!(!planner.commit(
        &selected,
        ResourceCost::new(30, 0, 0, 0),
        OBSERVED_AT_MS + 1_000,
    ));
    assert_eq!(planner.network_tokens(OBSERVED_AT_MS), 10);
}
