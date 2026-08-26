use crate::adaptive::{
    ActionKind, ActionNode, ActionValue, BeamConfig, HardBudget, ResourceCost, TransformKind,
    WarpSearch,
};
use crate::PostId;

#[test]
fn an_explicit_dependency_can_follow_its_whole_fetch() {
    let whole = node(1, ActionKind::FetchWhole { maximum_bytes: 1 }, 1, &[]);
    let transform = node(2, ActionKind::Transform(TransformKind::Remux), 100, &[1]);
    let decision = WarpSearch::new(BeamConfig::new(2, 8, 32, u64::MAX))
        .choose_first(&[whole, transform], HardBudget::unlimited());

    assert_eq!(
        decision.chosen_plan.expect("valid test fixture").action_ids,
        [1, 2]
    );
}

fn node(id: u16, kind: ActionKind, score: i64, requires: &[u16]) -> ActionNode {
    ActionNode::new(
        id,
        PostId::new("post"),
        kind,
        ActionValue::from_net_micros(score),
    )
    .with_resources(ResourceCost::new(1, 1, 1, 1))
    .with_origin("https://origin.example/media")
    .requiring(requires)
}
