use super::warp_backward_fixture::{context, starting_state};
use crate::adaptive::{
    AdaptivePlayabilityPolicy, InFlightAction, MediaLayout, PlannerLimits, PlayabilitySnapshot,
    WarpPlanner, WarpPlannerInput, WarpPlanningDecision, REQUEST_SLICE_BYTES,
};
use crate::origin_model::OriginModel;
use crate::{ActionId, ByteRange};

pub(super) fn state() -> PlayabilitySnapshot {
    let mut state = starting_state();
    state.candidates[1].origins[0].source = "https://independent.example/previous.mp4".into();
    let current = &mut state.candidates[0];
    current.layout = MediaLayout::Streamable;
    current.in_flight.push(InFlightAction::range(
        ActionId::new(7),
        ByteRange::new(0, 65_536),
        &current.origins[0].source,
        20_000,
        true,
    ));
    state
}

fn plan(state: &PlayabilitySnapshot, per_origin: u16) -> WarpPlanningDecision {
    let base = AdaptivePlayabilityPolicy.plan(state);
    let context = context(state).with_limits(PlannerLimits {
        network_burst_bytes: 2 * REQUEST_SLICE_BYTES,
        network_rate_bytes_per_second: 2 * REQUEST_SLICE_BYTES,
        cpu_ms: 0,
        request_tokens: 2,
        per_origin_requests: per_origin,
    });
    WarpPlanner::default().plan(WarpPlannerInput::new(
        state,
        &base,
        &OriginModel::default(),
        &context,
    ))
}

pub(super) fn history_admitted(state: &PlayabilitySnapshot, per_origin: u16) -> bool {
    let decision = plan(state, per_origin);
    decision.generated.actions.iter().any(|action| {
        action.node.post == state.candidates[1].post
            && action.node.resources.requests > 0
            && decision.admissible_action_ids.contains(&action.node.id)
    })
}
