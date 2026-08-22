use crate::adaptive::warp::budget::hard::{BudgetDenial, HardBudget, ResourceCost};
use crate::adaptive::{
    ActionKind, ActionNode, ActionValue, BeamConfig, HlsBootstrapStage, SegmentedStorageBudget,
    WarpSearch,
};
use crate::PostId;

const STAGE_BYTES: u64 = 8 * 1024 * 1024;

#[test]
fn four_stages_fit_and_the_fifth_is_pruned_from_shared_segmented_capacity() {
    let actions: Vec<_> = (1..=5).map(|id| hls(id, 1, &[])).collect();
    let budget = budget(4 * STAGE_BYTES);
    assert!(budget.clone().protect(&actions[..4]).is_some());
    assert!(budget.clone().protect(&actions).is_none());

    let mut consumed = budget;
    for action in &actions[..4] {
        assert_eq!(consumed.consume_action(action), Ok(()));
    }
    assert_eq!(
        consumed.consume_action(&actions[4]),
        Err(BudgetDenial::HardLimit)
    );
}

#[test]
fn hls_uses_segmented_capacity_without_consuming_progressive_storage() {
    let mut budget = budget(STAGE_BYTES);
    assert_eq!(budget.consume_action(&hls(1, 1, &[])), Ok(()));
    assert_eq!(budget.consume_action(&progressive(2)), Ok(()));
    assert_eq!(
        budget.consume_action(&progressive(3)),
        Err(BudgetDenial::HardLimit)
    );
}

#[test]
fn infeasible_fifth_stage_cannot_bias_the_first_action() {
    let mut actions = vec![progressive(10)];
    actions.extend((1..=5).map(|id| {
        let dependency = (id > 1).then_some(id - 1).into_iter().collect::<Vec<_>>();
        hls(id, if id == 5 { 100 } else { 1 }, &dependency)
    }));
    let search = WarpSearch::new(BeamConfig::new(5, 32, 512, u64::MAX));
    let bounded = search.choose_first(&actions, budget(4 * STAGE_BYTES));
    let expanded = search.choose_first(&actions, budget(5 * STAGE_BYTES));

    assert_eq!(bounded.action.map(|node| node.id), Some(10));
    assert_eq!(expanded.action.map(|node| node.id), Some(1));
}

fn budget(bytes: u64) -> HardBudget {
    HardBudget::new(ResourceCost::new(bytes, 1, 0, 0), 0)
        .with_segmented_storage(SegmentedStorageBudget::new(bytes))
}

fn hls(id: u16, score: i64, requires: &[u16]) -> ActionNode {
    ActionNode::new(
        id,
        PostId::new(format!("p{id}")),
        ActionKind::HlsBootstrap {
            stage: HlsBootstrapStage::Initialization,
            cursor: Default::default(),
            maximum_bytes: STAGE_BYTES,
        },
        ActionValue::from_net_micros(score),
    )
    .with_resources(ResourceCost::new(0, STAGE_BYTES, 0, 0))
    .requiring(requires)
}

fn progressive(id: u16) -> ActionNode {
    ActionNode::new(
        id,
        PostId::new("p1"),
        ActionKind::FetchWhole { maximum_bytes: 1 },
        ActionValue::from_net_micros(10),
    )
    .with_resources(ResourceCost::new(0, 1, 0, 0))
}
