use crate::adaptive::{
    AdaptivePlayabilityPolicy, FeedOffset, PlannerCommand, PlannerContext, PlayerPreparation,
    RetrievalRequest, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::ByteRange;

#[test]
fn retained_previous_video_has_a_bounded_startup_choice_before_focus_returns() {
    let mut input = snapshot(2, 20_000_000, 20_000, 0);
    input.navigation.backward_swipes_per_minute = 30;
    input.candidates[0].present = vec![ByteRange::new(0, 3_750_000)];
    input.candidates[0].finalized = true;
    input.candidates[1].feed_offset = FeedOffset::new(-1);
    input.candidates[1].view_probability = input.navigation.view_probability(FeedOffset::new(-1));
    input.candidates[1].player_preparation = PlayerPreparation::Unverified;
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let context = PlannerContext::explicitly_unavailable(&input);
    let origins = OriginModel::default();

    let result =
        WarpPlanner::default().plan(WarpPlannerInput::new(&input, &base, &origins, &context));

    assert!(
        result.generated.actions.iter().any(bounded_previous_start),
        "a likely reverse swipe needs a preparation choice before it becomes current"
    );
}

fn bounded_previous_start(action: &crate::adaptive::GeneratedAction) -> bool {
    matches!(&action.command,
        PlannerCommand::Transfer(work) if work.post.as_str() == "p1"
            && matches!(work.request, RetrievalRequest::FetchRange { bytes, .. }
                if bytes.start == 0 && bytes.end <= crate::adaptive::REQUEST_SLICE_BYTES))
}
