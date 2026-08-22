use super::warp_planner_commit_test_support::{planner, selected, OBSERVED_AT_MS};
use crate::adaptive::ResourceCost;

#[test]
fn a_smaller_committed_cap_consumes_its_exact_network_bytes() {
    let mut planner = planner(100, 0);
    let selected = selected(ResourceCost::new(100, 80, 5, 1));
    let committed = ResourceCost::new(30, 20, 2, 1);

    assert!(planner.commit(&selected, committed, OBSERVED_AT_MS));
    assert_eq!(planner.network_tokens(OBSERVED_AT_MS), 70);
}
