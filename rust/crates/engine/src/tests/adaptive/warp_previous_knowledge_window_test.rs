use super::warp_backward_fixture::{plan, state};
use crate::adaptive::{FeedOffset, MediaLayout, PlannerCommand, WarpPlannerConfig};

#[test]
fn retained_history_receives_metadata_preparation_before_focus_returns() {
    let mut state = state();
    let future = &mut state.candidates[1];
    future.feed_offset = FeedOffset::new(-3);
    future.view_probability = state.navigation.view_probability(future.feed_offset);
    future.layout = MediaLayout::Unknown;
    future.startup = None;
    let post = future.post.clone();

    let decision = plan(&state, WarpPlannerConfig::default());

    assert!(
        decision.selected.is_some_and(|action| {
            matches!(action.command, PlannerCommand::ProbeHead { post: selected, .. }
                if selected == post)
                && action.node.resources.requests == 1
                && action.node.resources.storage_bytes == 0
        }),
        "an eligible item in the retained history window needs bounded knowledge acquisition"
    );
}
