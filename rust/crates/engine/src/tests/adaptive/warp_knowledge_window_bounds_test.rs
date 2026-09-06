use super::warp_backward_fixture::{plan, state};
use crate::adaptive::{FeedOffset, MediaLayout, PlannerCommand, WarpPlannerConfig};

#[test]
fn metadata_preparation_stays_inside_the_bounded_navigation_window() {
    for (offset, expected) in [(-6, false), (-5, true), (24, true), (25, false)] {
        let mut state = state();
        let candidate = &mut state.candidates[1];
        candidate.feed_offset = FeedOffset::new(offset);
        candidate.layout = MediaLayout::Unknown;
        candidate.startup = None;
        let post = candidate.post.clone();
        let decision = plan(&state, WarpPlannerConfig::default());

        let offered = decision.generated.actions.iter().any(|action| {
            matches!(&action.command, PlannerCommand::ProbeHead { post: target, .. } if target == &post)
        });
        assert_eq!(offered, expected, "knowledge window offset {offset}");
    }
}
