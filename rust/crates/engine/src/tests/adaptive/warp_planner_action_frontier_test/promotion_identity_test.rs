use super::action;
use crate::adaptive::{ActionFrontier, ActionKind, ActionValue, EpsilonBuckets, ResourceCost};
use crate::ActionId;

#[test]
fn epsilon_pruning_preserves_a_live_promotion_identity() {
    let mut transfer = action(1, 16, 100, 1_000);
    transfer.kind = ActionKind::FetchWhole { maximum_bytes: 16 };
    transfer.value = ActionValue::from_net_micros(1_000_001);
    let mut promotion = action(2, 16, 100, 1_000);
    promotion.kind = ActionKind::Promote {
        active: ActionId::new(7),
        maximum_bytes: 16,
    };
    promotion.resources = ResourceCost::new(12, 12, 0, 0);
    let frontier = ActionFrontier::prune(
        vec![transfer.clone(), promotion.clone()],
        EpsilonBuckets::new(20, 4, 100, 100),
    );

    assert_eq!(frontier.retained, [transfer, promotion]);
    assert!(frontier.pruned_ids.is_empty());
}

#[test]
fn epsilon_pruning_preserves_distinct_promotion_targets() {
    let mut first = action(1, 16, 100, 1_000);
    first.kind = ActionKind::Promote {
        active: ActionId::new(7),
        maximum_bytes: 16,
    };
    let mut second = first.clone();
    second.id = 2;
    second.kind = ActionKind::Promote {
        active: ActionId::new(8),
        maximum_bytes: 20,
    };
    let frontier = ActionFrontier::prune(
        vec![first.clone(), second.clone()],
        EpsilonBuckets::new(20, 4, 100, 100),
    );

    assert_eq!(frontier.retained, [first, second]);
    assert!(frontier.pruned_ids.is_empty());
}
