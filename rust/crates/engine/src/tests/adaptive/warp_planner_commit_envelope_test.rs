use super::warp_planner_commit_test_support::{planner, selected, OBSERVED_AT_MS};
use crate::adaptive::ResourceCost;

#[test]
fn any_resource_above_the_selected_envelope_is_rejected_without_charge() {
    let envelope = ResourceCost::new(80, 60, 4, 1);
    let selected = selected(envelope);
    let excessive = [
        ResourceCost::new(81, 60, 4, 1),
        ResourceCost::new(80, 61, 4, 1),
        ResourceCost::new(80, 60, 5, 1),
        ResourceCost::new(80, 60, 4, 2),
    ];

    for committed in excessive {
        let mut planner = planner(100, 0);
        assert!(!planner.commit(&selected, committed, OBSERVED_AT_MS));
        assert_eq!(planner.network_tokens(OBSERVED_AT_MS), 100);
    }
}
