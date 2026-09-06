use super::warp_backward_fixture::{plan, state};
use crate::adaptive::{FeedOffset, MediaLayout, WarpPlannerConfig};

#[test]
fn expanded_metadata_preparation_preserves_retrieval_eligibility() {
    let mut state = state();
    let candidate = &mut state.candidates[1];
    candidate.feed_offset = FeedOffset::new(12);
    candidate.retrieval_eligible = false;
    candidate.layout = MediaLayout::Unknown;
    candidate.startup = None;
    let post = candidate.post.clone();
    let decision = plan(&state, WarpPlannerConfig::default());

    assert!(
        decision
            .generated
            .actions
            .iter()
            .all(|action| action.node.post != post),
        "the wider knowledge window cannot authorize an ineligible source"
    );
}
