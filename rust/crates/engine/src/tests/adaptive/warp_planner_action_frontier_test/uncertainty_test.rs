use super::action;
use crate::adaptive::axiom_test_support::ActionFrontier;
use crate::adaptive::EpsilonBuckets;

#[test]
fn summary_forecasts_do_not_prove_structural_dominance() {
    let actions = vec![action(1, 100, 100, 1_000), action(2, 100, 110, 1_000)];
    let frontier = ActionFrontier::prune(actions.clone(), EpsilonBuckets::disabled());
    assert_eq!(frontier.retained, actions);
    assert!(frontier.pruned_ids.is_empty());
}
