use super::action;
use crate::adaptive::axiom_test_support::ActionFrontier;
use crate::adaptive::EpsilonBuckets;

#[test]
fn structural_pruning_preserves_source_and_learning_provenance() {
    let first = action(1, 100, 100, 1_000).with_origin("https://one.example/video.mp4");
    let second = action(2, 100, 100, 1_000).with_origin("https://two.example/video.mp4");
    let slower = action(3, 100, 110, 1_000).with_origin("https://three.example/video.mp4");
    for epsilon in [
        EpsilonBuckets::disabled(),
        EpsilonBuckets::new(20, 16_384, 100, 100),
    ] {
        let actions = vec![first.clone(), second.clone(), slower.clone()];
        let frontier = ActionFrontier::prune(actions.clone(), epsilon);
        assert_eq!(frontier.retained, actions);
        assert!(frontier.pruned_ids.is_empty());
    }
}
