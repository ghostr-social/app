use crate::adaptive::{
    AdaptivePlayabilityPolicy, PlannerContext, ResourceFeedback, ResourceObservation, WarpPlanner,
    WarpPlannerConfig, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn planner_reserves_rescue_capacity_updates_prices_and_commits_one_action() {
    let input = snapshot(4, 8_000_000, 8_000, 20);
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let mut context =
        PlannerContext::explicitly_unavailable(&input).with_feedback(ResourceFeedback {
            revision: 1,
            actual: ResourceObservation::new(200, 120, 20, 4),
            target: ResourceObservation::new(100, 100, 10, 2),
            price_snapshot: None,
        });
    context.limits.request_tokens = 3;
    let config = WarpPlannerConfig::default().with_rescue_thresholds(2_000, 2_000);
    let mut planner = WarpPlanner::new(config);
    let decision = planner.plan(WarpPlannerInput::new(
        &input,
        &base,
        &OriginModel::default(),
        &context,
    ));

    assert!(decision.reserve.reserved_request_slots <= 1);
    assert!(decision.prices.network_micros > 0 && decision.prices.request_micros > 0);
    assert_eq!(
        decision.search.committed_actions,
        u8::from(decision.selected.is_some())
    );
    assert!(decision.search.committed_actions <= 1);
    assert_eq!(
        decision.common_random_seed,
        decision
            .evaluation
            .expect("valid test fixture")
            .common_random_seed
    );
    assert!(!decision.admissible_action_ids.is_empty());
    if let Some(selected) = &decision.selected {
        let before = planner.network_tokens(input.observed_at_ms);
        assert!(planner.commit(selected, selected.node.resources, input.observed_at_ms,));
        assert!(planner.network_tokens(input.observed_at_ms) <= before);
    }
}

#[test]
fn safety_without_chance_feasible_rescue_records_degraded_least_risk_choice() {
    let mut input = snapshot(2, 400_000, 8_000, 20);
    input
        .candidates
        .iter_mut()
        .for_each(|item| item.origins[0].failure_bps = 9_900);
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let context = PlannerContext::explicitly_unavailable(&input);
    let config = WarpPlannerConfig::default().with_rescue_thresholds(9_999, 9_999);
    let mut planner = WarpPlanner::new(config);
    let decision = planner.plan(WarpPlannerInput::new(
        &input,
        &base,
        &OriginModel::default(),
        &context,
    ));

    if base.mode != crate::adaptive::ControlMode::Normal {
        assert!(decision.reserve.degraded);
        assert!(decision.selected.is_some());
    }
}
