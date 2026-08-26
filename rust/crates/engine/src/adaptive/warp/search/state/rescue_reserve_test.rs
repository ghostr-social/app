use crate::adaptive::{
    ActionKind, ActionNode, ActionValue, BeamConfig, HardBudget, ResourceCost, SearchPruneReason,
    WarpSearch,
};
use crate::{ByteRange, PostId};

#[test]
fn beam_search_cannot_auction_the_pending_rescue_slot() {
    let ordinary = request(1, 100);
    let second = request(2, 90);
    let rescue = request(3, -1);
    let budget = HardBudget::new(ResourceCost::new(2, 2, 0, 2), 2)
        .protect(core::slice::from_ref(&rescue))
        .expect("valid test fixture");
    let decision = WarpSearch::new(BeamConfig::new(2, 8, 32, u64::MAX))
        .choose_first(&[ordinary, second, rescue], budget);

    assert_eq!(decision.action.expect("valid test fixture").id, 1);
    assert_eq!(
        decision.chosen_plan.expect("valid test fixture").action_ids,
        vec![1]
    );
    assert!(decision.pruned_plans.iter().any(|plan| {
        plan.action_ids == [1, 2] && plan.reason == SearchPruneReason::ReserveUnderflow
    }));
}

fn request(id: u16, score: i64) -> ActionNode {
    ActionNode::new(
        id,
        PostId::new(format!("p{id}")),
        ActionKind::FetchRange(ByteRange::new(u64::from(id), u64::from(id) + 1)),
        ActionValue::from_net_micros(score),
    )
    .with_resources(ResourceCost::new(1, 1, 0, 1))
    .with_origin(format!("https://p{id}.example/media"))
}
