use crate::adaptive::{
    ActionKind, AdaptivePlayabilityPolicy, MediaLayout, PlannerContext, WarpPlanner,
    WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn unresolved_visible_current_does_not_generate_head() {
    let mut input = snapshot(1, 20_000_000, 0, 0);
    input.candidates[0].layout = MediaLayout::Unknown;
    input.candidates[0].total_bytes = None;
    input.candidates[0].evidence = Default::default();
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let context = PlannerContext::explicitly_unavailable(&input);

    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &input,
        &base,
        &OriginModel::default(),
        &context,
    ));

    assert!(!decision.generated.actions.iter().any(|action| {
        action.node.post == input.playback.current && action.node.kind == ActionKind::Head
    }));
}
