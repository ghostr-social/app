use crate::adaptive::{
    AdaptivePlayabilityPolicy, FeedOffset, PlannerCommand, PlannerContext, PlannerLimits,
    PlayerPreparation, WarpPlanner, WarpPlannerInput, REQUEST_SLICE_BYTES,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::ByteRange;

#[test]
fn backward_preparation_is_admissible_with_a_single_bounded_spare_request() {
    let mut input = snapshot(2, 20_000_000, 20_000, 0);
    input.navigation.backward_swipes_per_minute = 30;
    input.candidates[0].present = vec![ByteRange::new(0, 3_750_000)];
    input.candidates[0].finalized = true;
    input.candidates[1].feed_offset = FeedOffset::new(-1);
    input.candidates[1].view_probability = input.navigation.view_probability(FeedOffset::new(-1));
    input.candidates[1].player_preparation = PlayerPreparation::Unverified;
    let base = AdaptivePlayabilityPolicy.plan(&input);
    let context = PlannerContext::explicitly_unavailable(&input).with_limits(PlannerLimits {
        network_burst_bytes: REQUEST_SLICE_BYTES,
        network_rate_bytes_per_second: REQUEST_SLICE_BYTES,
        cpu_ms: 0,
        request_tokens: 1,
        per_origin_requests: 1,
    });
    let origins = OriginModel::default();

    let result =
        WarpPlanner::default().plan(WarpPlannerInput::new(&input, &base, &origins, &context));

    let chosen = result
        .generated
        .actions
        .iter()
        .find(|action| result.admissible_action_ids.contains(&action.node.id))
        .expect("a bounded prior dependency is admissible within the spare budget");
    assert!(matches!(&chosen.command, PlannerCommand::Transfer(work)
        if work.post.as_str() == "p1"));
    assert!(chosen.node.authorized_resources().network_bytes <= REQUEST_SLICE_BYTES);
    assert_eq!(chosen.node.authorized_resources().requests, 1);
}
