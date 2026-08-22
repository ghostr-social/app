use crate::adaptive::{
    ActivePlannerContext, AdaptivePlayabilityPolicy, PlannerCommand, PlannerContext, WarpPlanner,
    WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::{ActionId, PostId};

#[test]
fn detached_active_action_is_semantically_admissible_and_selected_for_cancel() {
    let mut input = snapshot(1, 8_000_000, 20_000, 20);
    input
        .candidates
        .iter_mut()
        .for_each(|candidate| candidate.retrieval_eligible = false);
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let action = ActionId::new(41);
    let active = ActivePlannerContext::new(action, PostId::new("detached"))
        .with_continuation_advantage(-100_000);
    let context = PlannerContext::explicitly_unavailable(&input).with_active(active);
    let mut planner = WarpPlanner::default();

    let decision = planner.plan(WarpPlannerInput::new(
        &input,
        &base,
        &OriginModel::default(),
        &context,
    ));

    let selected = decision.selected.expect("detached cancel selected");
    assert_eq!(selected.command, PlannerCommand::Cancel(action));
    assert!(decision.admissible_action_ids.contains(&selected.node.id));
}
