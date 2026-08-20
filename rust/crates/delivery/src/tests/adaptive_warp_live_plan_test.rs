use crate::tests::adaptive_plan_support::plan;
use ghostr_engine::adaptive::{PlannerCommand, StorageSnapshot};

#[test]
fn live_plan_exposes_warp_decision_and_only_its_first_new_transfer() {
    let work = plan(2_000, 2_000_000, StorageSnapshot::new(2_000_000_000, 0));
    let decision = work.warp.as_ref().expect("advanced WARP decision");
    let selected = decision.selected.as_ref().expect("selected WARP action");
    let expects_transfer = matches!(
        selected.command,
        PlannerCommand::Transfer(_) | PlannerCommand::Hedge { .. }
    );

    assert_eq!(work.selected_transfers.len(), usize::from(expects_transfer));
    assert_eq!(decision.search.committed_actions, 1);
    assert!(!decision.search.retained_plans.is_empty());
}
