use super::action;
use crate::adaptive::axiom_test_support::ActionFrontier;
use crate::adaptive::{ActionKind, EpsilonBuckets};
use crate::ByteRange;

#[test]
fn structural_pruning_preserves_distinct_byte_effects() {
    let cheap = action(1, 100, 100, 1_000);
    let mut demanded = action(2, 11_000, 110, 990);
    demanded.kind = ActionKind::FetchRange(ByteRange::new(9_000, 20_000));
    for epsilon in [
        EpsilonBuckets::disabled(),
        EpsilonBuckets::new(20, 16_384, 100, 100),
    ] {
        let actions = vec![cheap.clone(), demanded.clone()];
        let frontier = ActionFrontier::prune(actions.clone(), epsilon);
        assert_eq!(frontier.retained, actions);
        assert!(frontier.pruned_ids.is_empty());
    }
}
