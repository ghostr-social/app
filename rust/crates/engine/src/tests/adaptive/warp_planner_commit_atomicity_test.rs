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

#[test]
fn committed_admission_risk_is_restored_only_by_configured_refill() {
    let mut planner = planner(100, 10);
    let selected = selected(ResourceCost::new(100, 0, 0, 0));

    assert!(planner.commit(&selected, ResourceCost::new(90, 0, 0, 0), OBSERVED_AT_MS,));
    assert_eq!(planner.network_tokens(OBSERVED_AT_MS), 10);
    assert_eq!(planner.network_tokens(OBSERVED_AT_MS + 1_000), 20);
}

#[test]
fn adaptation_reset_preserves_tokens_reserved_by_live_execution() {
    let mut planner = planner(100, 0);
    let selected = selected(ResourceCost::new(100, 0, 0, 0));
    assert!(planner.commit(&selected, ResourceCost::new(80, 0, 0, 0), OBSERVED_AT_MS,));

    planner.reset_adaptation();

    assert_eq!(planner.network_tokens(OBSERVED_AT_MS), 20);
    planner.reconcile_network_reservation(80, 20, OBSERVED_AT_MS);
    assert_eq!(planner.network_tokens(OBSERVED_AT_MS), 80);
}
