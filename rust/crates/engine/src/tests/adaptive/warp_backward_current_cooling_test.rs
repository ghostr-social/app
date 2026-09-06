use super::warp_backward_fixture::{context, starting_state};
use crate::adaptive::{
    AdaptivePlayabilityPolicy, PlannerRetryAvailability, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;

#[test]
fn unavailable_current_retry_does_not_reserve_a_slot_against_history_preparation() {
    let state = starting_state();
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let context = context(&state).with_retry_availability(
        &state.playback.current,
        PlannerRetryAvailability::Cooling {
            eligible_at_ms: 20_000,
        },
    );
    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &context,
    ));
    let history = &state.candidates[1].post;
    assert!(decision
        .generated
        .actions
        .iter()
        .filter(|action| &action.node.post == history)
        .any(|action| decision.admissible_action_ids.contains(&action.node.id)));
}
