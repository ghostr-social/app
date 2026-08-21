use crate::adaptive::{
    ActionForecast, ActionFrontier, ActionKind, ActionNode, ActionValue, CompletionTimes,
    EpsilonBuckets, ResourceCost,
};
use crate::{ByteRange, PostId};

#[path = "warp_planner_action_frontier_test/dependency_test.rs"]
mod dependency_test;

fn action(id: u16, bytes: u64, p95: u64, gain: u64) -> ActionNode {
    ActionNode::new(
        id,
        PostId::new("video"),
        ActionKind::FetchRange(ByteRange::new(0, bytes)),
        ActionValue::from_net_micros(1_000_000),
    )
    .with_resources(ResourceCost::new(bytes, bytes, 0, 1))
    .with_forecast(ActionForecast::new(
        CompletionTimes::new(p95 / 2, p95, p95 * 2, p95 * 3),
        9_000,
        gain,
    ))
}

#[test]
fn exact_action_frontier_removes_only_monotonically_dominated_work() {
    let efficient = action(1, 64_000, 100, 2_000);
    let dominated = action(2, 128_000, 200, 2_000);
    let broader = action(3, 256_000, 300, 6_000);
    let frontier = ActionFrontier::prune(
        vec![dominated, broader.clone(), efficient.clone()],
        EpsilonBuckets::disabled(),
    );
    assert_eq!(frontier.retained, vec![efficient, broader]);
    assert_eq!(frontier.pruned_ids, vec![2]);
}

#[test]
fn action_epsilon_buckets_are_configurable_and_deterministic() {
    let first = action(1, 64_000, 100, 2_000);
    let near = action(2, 70_000, 110, 2_050);
    let frontier = ActionFrontier::prune(
        vec![near, first.clone()],
        EpsilonBuckets::new(20, 16_384, 100, 100),
    );
    assert_eq!(frontier.retained, vec![first]);
    assert_eq!(frontier.pruned_ids, vec![2]);
}
