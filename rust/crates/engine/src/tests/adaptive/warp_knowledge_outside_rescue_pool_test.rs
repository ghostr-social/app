use super::warp_backward_fixture::{context, state};
use crate::adaptive::{
    AdaptivePlayabilityPolicy, FeedOffset, MediaLayout, PlannerCommand, PlayerPreparation,
    SemanticScore, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::{ByteRange, PostId};

#[test]
fn distant_metadata_does_not_require_admission_to_the_rescue_reordering_pool() {
    let mut state = state();
    let future = &mut state.candidates[1];
    future.feed_offset = FeedOffset::new(12);
    future.view_probability = state.navigation.view_probability(future.feed_offset);
    future.layout = MediaLayout::Unknown;
    future.startup = None;
    let post = future.post.clone();
    let mut ready = state.candidates[0].clone();
    ready.post = PostId::new("ready-neighbor");
    ready.feed_offset = FeedOffset::new(1);
    ready.finalized = false;
    ready.present = vec![ByteRange::new(0, 65_536)];
    ready.player_preparation = PlayerPreparation::FirstFrameRendered;
    state.candidates.insert(1, ready);
    let context = context(&state).with_semantic(&post, SemanticScore::Unavailable { rank: 12 });
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &context,
    ));

    assert!(decision.generated.actions.iter().any(|action| {
        matches!(&action.command, PlannerCommand::ProbeHead { post: selected, .. } if selected == &post)
            && decision.admissible_action_ids.contains(&action.node.id)
    }), "metadata preparation must extend beyond the rescue reordering pool");
}
