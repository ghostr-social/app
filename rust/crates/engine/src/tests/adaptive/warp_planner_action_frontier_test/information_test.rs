use super::action;
use crate::adaptive::axiom_test_support::ActionFrontier;
use crate::adaptive::EpsilonBuckets;

#[test]
fn epsilon_pruning_keeps_distinct_information_outcomes() {
    let delivery = action(1, 100, 100, 1_000);
    let mut discovery = action(2, 100, 100, 1_000);
    discovery.value.delay_loss_micros = 0;
    discovery.value.information_value_micros = 1_000_000;
    let actions = vec![delivery, discovery];
    let frontier =
        ActionFrontier::prune(actions.clone(), EpsilonBuckets::new(20, 16_384, 100, 100));
    assert_eq!(frontier.retained, actions);
    assert!(frontier.pruned_ids.is_empty());
}
