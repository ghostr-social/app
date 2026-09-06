use super::warp_backward_fixture::{plan, state};
use crate::adaptive::PlayerPreparation;
use crate::playback::PlaybackPhase;

#[test]
fn likely_backward_navigation_cannot_take_the_only_current_startup_request() {
    let mut state = state();
    state.playback.phase = PlaybackPhase::Starting;
    state.playback.buffer_ahead_ms = 0;
    state.candidates[0].present.clear();
    state.candidates[0].finalized = false;
    state.candidates[0].player_preparation = PlayerPreparation::Unverified;
    let result = plan(&state, Default::default());
    let selected = result.selected.expect("current startup is feasible");
    assert_eq!(selected.node.post, state.playback.current);
    assert_eq!(selected.node.authorized_resources().requests, 1);
}
