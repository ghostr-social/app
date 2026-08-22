use crate::adaptive::{
    ActionKind, ActionNode, ActionValue, BeamConfig, HardBudget, SearchPruneReason, WarpSearch,
};
use crate::{ByteRange, PostId};
use std::time::Duration;

#[test]
fn latency_limit_is_checked_before_each_scorer_evaluation() {
    let nodes: Vec<_> = (1..=4).map(node).collect();
    let mut calls = 0;
    let mut scorer = |_actions: &[ActionNode]| {
        calls += 1;
        std::thread::sleep(Duration::from_millis(5));
        1
    };

    let decision = WarpSearch::new(BeamConfig::new(3, 8, 32, 1_000)).choose_first_recorded(
        &nodes,
        HardBudget::unlimited(),
        &mut scorer,
    );

    assert_eq!(calls, 1);
    assert!(decision.used_greedy_fallback);
    assert!(decision
        .pruned_plans
        .iter()
        .any(|plan| plan.reason == SearchPruneReason::PlannerLatency));
}

fn node(id: u16) -> ActionNode {
    ActionNode::new(
        id,
        PostId::new(format!("p{id}")),
        ActionKind::FetchRange(ByteRange::new(0, 64_000)),
        ActionValue::from_net_micros(i64::from(id)),
    )
}
