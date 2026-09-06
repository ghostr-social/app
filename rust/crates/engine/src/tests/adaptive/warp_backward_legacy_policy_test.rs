use super::warp_backward_fixture::{plan, state};
use crate::adaptive::{NavigationPreparationPolicy, WarpPlannerConfig};

#[test]
fn legacy_forward_only_policy_does_not_acquire_or_admit_history() {
    let state = state();
    let config = WarpPlannerConfig {
        navigation_preparation_policy: NavigationPreparationPolicy::ForwardOnly,
        ..Default::default()
    };
    let result = plan(&state, config);
    assert!(result
        .generated
        .actions
        .iter()
        .all(|action| action.node.post != state.candidates[1].post));
    assert!(result.admissible_action_ids.is_empty());
}
