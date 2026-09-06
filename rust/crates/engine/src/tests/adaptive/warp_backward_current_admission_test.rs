use super::warp_backward_fixture::{plan, state};
use crate::adaptive::{MediaLayout, PlayerPreparation};
use crate::playback::PlaybackPhase;

#[test]
fn history_cannot_acquire_the_only_slot_while_current_bootstrap_is_missing() {
    let mut state = state();
    state.playback.phase = PlaybackPhase::Starting;
    state.playback.buffer_ahead_ms = 0;
    let current = &mut state.candidates[0];
    current.present.clear();
    current.finalized = false;
    current.layout = MediaLayout::Unknown;
    current.player_preparation = PlayerPreparation::Unverified;
    let decision = plan(&state, Default::default());
    let previous = &state.candidates[1].post;
    assert!(decision
        .generated
        .actions
        .iter()
        .any(|action| &action.node.post == previous));
    assert!(decision
        .generated
        .actions
        .iter()
        .filter(|action| &action.node.post == previous)
        .all(|action| !decision.admissible_action_ids.contains(&action.node.id)));
}
