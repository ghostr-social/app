use super::action;
use crate::adaptive::axiom_test_support::ActionFrontier;
use crate::adaptive::{
    ActionForecast, ActionKind, ActionNode, ActionValue, CompletionTimes, EpsilonBuckets,
    ResourceCost, TransformKind,
};
use crate::PostId;

#[test]
fn a_retained_action_keeps_its_dominated_prerequisite() {
    let prerequisite = action(1, 128_000, 200, 2_000);
    let dominant = action(2, 64_000, 100, 2_000);
    let dependent = ActionNode::new(
        3,
        PostId::new("video"),
        ActionKind::Transform(TransformKind::Remux),
        ActionValue::from_net_micros(2_000_000),
    )
    .with_resources(ResourceCost::new(0, 1, 1, 0))
    .with_forecast(ActionForecast::new(
        CompletionTimes::new(10, 10, 10, 10),
        10_000,
        6_000,
    ))
    .requiring(&[1]);
    let frontier = ActionFrontier::prune(
        vec![prerequisite.clone(), dominant.clone(), dependent.clone()],
        EpsilonBuckets::disabled(),
    );

    assert_eq!(frontier.retained, [prerequisite, dominant, dependent]);
    assert!(frontier.pruned_ids.is_empty());
}
