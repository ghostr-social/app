use super::warp_backward_fixture::{plan, state};
use crate::adaptive::{
    FeedOffset, MediaLayout, NavigationPreparationPolicy, PlannerCommand, WarpPlannerConfig,
};

#[test]
fn recorded_bidirectional_policy_keeps_its_original_metadata_window() {
    for offset in [-3, 12] {
        let mut state = state();
        let candidate = &mut state.candidates[1];
        candidate.feed_offset = FeedOffset::new(offset);
        candidate.layout = MediaLayout::Unknown;
        candidate.startup = None;
        let post = candidate.post.clone();
        let config = WarpPlannerConfig {
            navigation_preparation_policy: NavigationPreparationPolicy::Bidirectional,
            ..WarpPlannerConfig::default()
        };
        let decision = plan(&state, config);

        assert!(decision.generated.actions.iter().all(|action| {
            !matches!(&action.command, PlannerCommand::ProbeHead { post: target, .. } if target == &post)
        }), "older recorded policy must retain its choices at offset {offset}");
    }
}
