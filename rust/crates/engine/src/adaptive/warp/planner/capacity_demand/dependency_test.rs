use super::has_marginal_demand;
use crate::adaptive::{
    ActionKind, ActionNode, ActionValue, ResourceCost, RetainedSearchPlan, SearchDecision,
};
use crate::{ByteRange, PostId};

#[test]
fn dependent_future_request_does_not_claim_a_parallel_slot() {
    let root = request(1, &[]);
    let dependent = request(2, &[1]);
    let search = SearchDecision {
        chosen_plan: Some(RetainedSearchPlan {
            action_ids: vec![1, 2],
            score_micros: 10,
        }),
        ..SearchDecision::default()
    };

    assert!(!has_marginal_demand(&search, &[root, dependent], 1));
}

fn request(id: u16, requires: &[u16]) -> ActionNode {
    ActionNode::new(
        id,
        PostId::new("post"),
        ActionKind::FetchRange(ByteRange::new(u64::from(id), u64::from(id) + 1)),
        ActionValue::from_net_micros(10),
    )
    .with_resources(ResourceCost::new(1, 1, 0, 1))
    .with_origin("https://origin.example/media")
    .requiring(requires)
}
