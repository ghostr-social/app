use super::warp_backward_fixture::{plan, state};

#[test]
fn history_retained_only_for_eviction_cannot_acquire_payload() {
    let mut state = state();
    state.candidates[1].retrieval_eligible = false;
    let result = plan(&state, Default::default());
    assert!(result
        .generated
        .actions
        .iter()
        .all(|action| action.node.post != state.candidates[1].post));
    assert!(result.admissible_action_ids.is_empty());
}
