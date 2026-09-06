use super::warp_backward_fixture::{context, starting_state};
use crate::adaptive::{
    AdaptivePlayabilityPolicy, PlannerLimits, WarpPlanner, WarpPlannerInput, REQUEST_SLICE_BYTES,
};
use crate::origin_model::OriginModel;

pub(super) fn admitted_on(source: &str) -> bool {
    let mut state = starting_state();
    state.candidates[1].origins[0].source = source.into();
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let context = context(&state).with_limits(PlannerLimits {
        network_burst_bytes: 2 * REQUEST_SLICE_BYTES,
        network_rate_bytes_per_second: 2 * REQUEST_SLICE_BYTES,
        cpu_ms: 0,
        request_tokens: 2,
        per_origin_requests: 1,
    });
    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &context,
    ));
    decision
        .generated
        .actions
        .iter()
        .filter(|action| action.node.post == state.candidates[1].post)
        .any(|action| decision.admissible_action_ids.contains(&action.node.id))
}
