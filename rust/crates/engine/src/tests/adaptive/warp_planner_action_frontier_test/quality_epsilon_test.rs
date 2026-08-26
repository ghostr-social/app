use super::action;
use crate::adaptive::axiom_test_support::ActionFrontier;
use crate::adaptive::EpsilonBuckets;

#[test]
fn epsilon_pruning_keeps_a_material_quality_difference() {
    let mut lower = action(1, 64_000, 100, 2_000);
    lower.forecast = lower.forecast.with_quality(100_000);
    let mut higher = action(2, 70_000, 100, 2_000);
    higher.forecast = higher.forecast.with_quality(200_001);

    let frontier = ActionFrontier::prune(
        vec![lower.clone(), higher.clone()],
        EpsilonBuckets::new(20, 16_384, 100, 100_000),
    );

    assert_eq!(frontier.retained, [lower, higher]);
    assert!(frontier.pruned_ids.is_empty());
}
