use crate::adaptive::{
    AdaptivePlayabilityPolicy, FeedOffset, PlannerContext, ViewProbability, WarpPlanner,
    WarpPlannerConfig, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

#[test]
fn degraded_emergency_fallback_selects_only_playback_current_work() {
    let input = current_second_snapshot();
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let context = PlannerContext::explicitly_unavailable(&input);
    let config = WarpPlannerConfig::default().with_rescue_thresholds(9_999, 9_999);
    let decision = WarpPlanner::new(config).plan(WarpPlannerInput::new(
        &input,
        &base,
        &OriginModel::default(),
        &context,
    ));

    assert!(decision.reserve.degraded, "{:#?}", decision.reserve);
    let selected = decision.selected.expect("least-risk emergency action");
    assert_eq!(selected.node.post, input.playback.current);
}

fn current_second_snapshot() -> crate::adaptive::PlayabilitySnapshot {
    let mut input = snapshot(2, 20_000_000, 0, 20);
    input.playback.current = PostId::new("p1");
    input.candidates[0].feed_offset = FeedOffset::new(-1);
    input.candidates[0].view_probability = ViewProbability::new(0.1).unwrap();
    input.candidates[1].feed_offset = FeedOffset::new(0);
    input.candidates[1].view_probability = ViewProbability::new(1.0).unwrap();
    input
}
