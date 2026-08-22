use super::super::reserve_support;
use crate::adaptive::{
    DecisionRecord, ResourceCost, WarpPlanner, WarpPlannerInput, WarpPlanningDecision,
};

pub(super) fn planned() -> (crate::adaptive::PlayabilitySnapshot, WarpPlanningDecision) {
    reserve_support::planned()
}

pub(super) fn planned_network_boundary(
) -> (crate::adaptive::PlayabilitySnapshot, WarpPlanningDecision) {
    let mut state = reserve_support::rescue_state();
    let base = reserve_support::safety_plan();
    let mut context = reserve_support::rescue_context(&state);
    let origins = reserve_support::reliable_origin();
    context.limits.network_burst_bytes = 250_000;
    context.limits.network_rate_bytes_per_second = 125_000;
    let mut planner = WarpPlanner::new(reserve_support::replay_config());
    let initial = planner.plan(WarpPlannerInput::new(&state, &base, &origins, &context));
    let transfer = initial
        .generated
        .actions
        .iter()
        .find(|action| action.node.resources.network_bytes >= 125_000)
        .expect("fixture has a range transfer");
    assert!(planner.commit(transfer, ResourceCost::new(125_000, 0, 0, 0), 10_000));
    state.observed_at_ms = 11_000;
    let decision = planner.plan(WarpPlannerInput::new(&state, &base, &origins, &context));
    (state, decision)
}

pub(super) fn record(
    state: &crate::adaptive::PlayabilitySnapshot,
    decision: &WarpPlanningDecision,
) -> DecisionRecord {
    reserve_support::record(state, decision)
}

pub(super) fn capsule(
    decision: &mut WarpPlanningDecision,
) -> &mut crate::adaptive::PlannerReplayCapsule {
    decision
        .planner_replay
        .as_mut()
        .expect("real planner captures replay input")
}
