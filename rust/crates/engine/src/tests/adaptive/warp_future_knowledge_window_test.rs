use super::warp_backward_fixture::{plan, state};
use crate::adaptive::{FeedOffset, MediaLayout, PlannerCommand, WarpPlannerConfig};

#[test]
fn eligible_future_video_receives_metadata_preparation_before_it_is_nearby() {
    let mut state = state();
    let future = &mut state.candidates[1];
    future.feed_offset = FeedOffset::new(12);
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
        "an eligible item in the retained future window needs bounded knowledge acquisition"
    );
}
