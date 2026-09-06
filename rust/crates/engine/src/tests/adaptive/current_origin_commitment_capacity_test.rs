use super::current_origin_commitment_fixture::contended_state;
use crate::adaptive::{AdaptivePlayabilityPolicy, FeedOffset};
use crate::{ActionId, PostId};

#[test]
fn origin_capacity_retains_only_the_future_work_that_fits_beside_current() {
    let mut input = contended_state();
    input.network.connection_capacity = 3;
    input.network.connection_ceiling = 3;
    input.network.per_authority_request_limit = 2;
    let mut second = input.candidates[1].clone();
    second.post = PostId::new("p2");
    second.feed_offset = FeedOffset::new(2);
    second.in_flight[0].action_id = ActionId::new(8);
    input.candidates.push(second);

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert_eq!(plan.retained.len(), 1);
    assert_eq!(plan.retained[0].action_id, ActionId::new(7));
    assert!(plan
        .allocations
        .iter()
        .any(|work| work.post == input.playback.current));
}
