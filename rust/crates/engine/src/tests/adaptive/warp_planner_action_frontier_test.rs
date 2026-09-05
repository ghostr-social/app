use crate::adaptive::axiom_test_support::ActionFrontier;
use crate::adaptive::{EpsilonBuckets, ResourceCost};

#[path = "warp_planner_action_frontier_test/test_support.rs"]
mod test_support;
use test_support::action;

#[path = "warp_planner_action_frontier_test/dependency_test.rs"]
mod dependency_test;
#[path = "warp_planner_action_frontier_test/information_test.rs"]
mod information_test;
#[path = "warp_planner_action_frontier_test/promotion_identity_test.rs"]
mod promotion_identity_test;
#[path = "warp_planner_action_frontier_test/quality_epsilon_test.rs"]
mod quality_epsilon_test;
#[path = "warp_planner_action_frontier_test/quality_pareto_test.rs"]
mod quality_pareto_test;
#[path = "warp_planner_action_frontier_test/range_identity_test.rs"]
mod range_identity_test;
#[path = "warp_planner_action_frontier_test/requirement_identity_test.rs"]
mod requirement_identity_test;
#[path = "warp_planner_action_frontier_test/source_identity_test.rs"]
mod source_identity_test;
#[path = "warp_planner_action_frontier_test/uncertainty_test.rs"]
mod uncertainty_test;

#[test]
fn exact_action_frontier_removes_only_monotonically_dominated_work() {
    let efficient = action(1, 64_000, 100, 2_000);
    let dominated =
        action(2, 64_000, 100, 2_000).with_resources(ResourceCost::new(128_000, 128_000, 0, 1));
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
    let near =
        action(2, 64_000, 110, 2_050).with_resources(ResourceCost::new(70_000, 64_000, 0, 1));
    let frontier = ActionFrontier::prune(
        vec![near, first.clone()],
        EpsilonBuckets::new(20, 16_384, 100, 100),
    );
    assert_eq!(frontier.retained, vec![first]);
    assert_eq!(frontier.pruned_ids, vec![2]);
}
