use super::warp_backward_fixture::{plan, starting_state};
use crate::adaptive::{FeedOffset, MediaLayout, WarpPlannerConfig};

#[test]
fn distant_metadata_cannot_take_the_only_current_startup_request() {
    let mut state = starting_state();
    let future = &mut state.candidates[1];
    future.feed_offset = FeedOffset::new(12);
    future.view_probability = state.navigation.view_probability(future.feed_offset);
    future.layout = MediaLayout::Unknown;
    future.startup = None;

    let decision = plan(&state, WarpPlannerConfig::default());
    let selected = decision.selected.expect("current startup is feasible");

    assert_eq!(
        selected.node.post, state.playback.current,
        "distant knowledge acquisition cannot starve the current video"
    );
}
