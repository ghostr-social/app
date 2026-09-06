use super::action;
use crate::adaptive::axiom_test_support::ActionFrontier;
use crate::adaptive::EpsilonBuckets;

#[test]
fn structural_pruning_preserves_distinct_dependency_paths() {
    let independent = action(1, 100, 100, 1_000);
    let dependent = action(2, 100, 110, 1_000).requiring(&[1]);
    for epsilon in [
        EpsilonBuckets::disabled(),
        EpsilonBuckets::new(20, 16_384, 100, 100),
    ] {
        let actions = vec![independent.clone(), dependent.clone()];
        let frontier = ActionFrontier::prune(actions.clone(), epsilon);
        assert_eq!(frontier.retained, actions);
        assert!(frontier.pruned_ids.is_empty());
    }
}
