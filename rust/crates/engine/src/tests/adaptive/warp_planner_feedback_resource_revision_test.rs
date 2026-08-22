use crate::adaptive::{
    AdaptivePlayabilityPolicy, PlannerContext, ResourceFeedback, ResourceObservation, WarpPlanner,
    WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn within_interval_changes_wait_for_the_next_controller_sample() {
    let input = snapshot(2, 8_000_000, 8_000, 20);
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let mut planner = WarpPlanner::default();
    let once = plan(&mut planner, &input, &base, feedback(200, 2));
    let request_changed = plan(&mut planner, &input, &base, feedback(300, 4));

    assert_eq!(
        request_changed.prices.network_micros,
        once.prices.network_micros
    );
    assert_eq!(
        request_changed.prices.request_micros,
        once.prices.request_micros
    );
}

fn plan(
    planner: &mut WarpPlanner,
    input: &crate::adaptive::PlayabilitySnapshot,
    base: &crate::adaptive::AllocationPlan,
    feedback: ResourceFeedback,
) -> crate::adaptive::WarpPlanningDecision {
    let context = PlannerContext::explicitly_unavailable(input).with_feedback(feedback);
    planner.plan(WarpPlannerInput::new(
        input,
        base,
        &OriginModel::default(),
        &context,
    ))
}

fn feedback(network: u64, requests: u64) -> ResourceFeedback {
    ResourceFeedback {
        revision: 7,
        actual: ResourceObservation::new(network, 0, 0, requests),
        target: ResourceObservation::new(100, 1, 0, 1),
        price_snapshot: None,
    }
}
