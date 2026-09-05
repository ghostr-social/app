use crate::adaptive::{
    AdaptivePlayabilityPolicy, PlannerContext, ResourceFeedback, ResourceFeedbackCursor,
    ResourceObservation, ResourcePriceSnapshot, ResourcePrices, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn authoritative_prices_ignore_equal_and_stale_cursors_across_wrap() {
    let state = snapshot(2, 8_000_000, 8_000, 20);
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let mut planner =
        WarpPlanner::new(crate::adaptive::WarpPlannerConfig::default().with_lookahead());
    let first = plan(&mut planner, &state, &base, feedback(0, 2, 200));
    let stale = plan(&mut planner, &state, &base, feedback(0, 1, 900));
    let equal = plan(&mut planner, &state, &base, feedback(0, 2, 800));
    let wrapped = plan(&mut planner, &state, &base, feedback(1, 0, 300));

    assert_eq!(stale.prices, first.prices);
    assert_eq!(equal.prices, first.prices);
    assert_eq!(wrapped.prices.network_micros, 300);
}

#[test]
fn legacy_feedback_ignores_a_lower_revision() {
    let state = snapshot(2, 8_000_000, 8_000, 20);
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let mut planner =
        WarpPlanner::new(crate::adaptive::WarpPlannerConfig::default().with_lookahead());
    let first = plan(&mut planner, &state, &base, legacy(9));
    let stale = plan(&mut planner, &state, &base, legacy(8));

    assert_eq!(stale.prices, first.prices);
}

fn plan(
    planner: &mut WarpPlanner,
    state: &crate::adaptive::PlayabilitySnapshot,
    base: &crate::adaptive::AllocationPlan,
    feedback: ResourceFeedback,
) -> crate::adaptive::WarpPlanningDecision {
    let context = PlannerContext::explicitly_unavailable(state).with_feedback(feedback);
    planner.plan(WarpPlannerInput::new(
        state,
        base,
        &OriginModel::default(),
        &context,
    ))
}

fn feedback(epoch: u64, revision: u64, network_micros: u64) -> ResourceFeedback {
    ResourceFeedback::authoritative(
        ResourcePriceSnapshot::new(
            ResourceFeedbackCursor::new(epoch, revision),
            ResourcePrices {
                network_micros,
                ..ResourcePrices::default()
            },
        ),
        ResourceObservation::default(),
        ResourceObservation::default(),
    )
}

fn legacy(revision: u64) -> ResourceFeedback {
    ResourceFeedback {
        revision,
        actual: ResourceObservation::new(200, 0, 0, 0),
        target: ResourceObservation::new(100, 1, 0, 1),
        price_snapshot: None,
    }
}
