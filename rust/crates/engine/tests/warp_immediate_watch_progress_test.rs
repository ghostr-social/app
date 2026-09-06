use crate::adaptive::{
    AdaptivePlayabilityPolicy, PlannerContext, PlannerWatchEvidence, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::ByteRange;

#[test]
fn an_immediate_next_watch_still_receives_feasible_preparation() {
    let mut input = snapshot(2, 20_000_000, 20_000, 0);
    input.candidates[0].present = vec![ByteRange::new(0, 3_750_000)];
    let ahead = input.candidates[1].post.clone();
    let context = PlannerContext::explicitly_unavailable(&input).with_watch(
        &ahead,
        PlannerWatchEvidence::learned(4_200, 0, 0, 0, 10_000, None),
    );
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &input,
        &base,
        &OriginModel::default(),
        &context,
    ));
    let action = decision
        .selected
        .expect("spare capacity prepares the next video");
    assert_eq!(
        action.node.post, ahead,
        "a missed deadline must not cause idling"
    );
    assert!(
        action.node.resources.network_bytes > 0,
        "prepare useful media"
    );
}
