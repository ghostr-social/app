use crate::adaptive::{
    AdaptivePlayabilityPolicy, DecisionPrivacy, DecisionRecord, FeedOffset, PlannerContext,
    ShadowPrices, WarpDecisionRecordInput, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn backward_preparation_policy_is_explicit_in_recorded_planner_inputs() {
    let mut state = snapshot(2, 20_000_000, 20_000, 0);
    state.candidates[1].feed_offset = FeedOffset::new(-1);
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let context = PlannerContext::explicitly_unavailable(&state);
    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &context,
    ));
    let record = DecisionRecord::capture_warp(WarpDecisionRecordInput {
        sequence: 1,
        snapshot: &state,
        decision: &decision,
        legacy_shadow_prices: ShadowPrices::default(),
        models: &[],
        privacy: &DecisionPrivacy::from_key([17; 32]),
    });
    let json = serde_json::to_value(record).expect("record JSON");
    assert_eq!(
        json["warp_decision"]["planner_replay_capsule"]["config"]["navigation_preparation_policy"],
        "bidirectional_window"
    );
}
