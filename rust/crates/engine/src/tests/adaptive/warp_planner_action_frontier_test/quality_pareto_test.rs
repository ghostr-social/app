use super::action;
use crate::adaptive::axiom_test_support::ActionFrontier;
use crate::adaptive::EpsilonBuckets;

#[test]
fn exact_pruning_keeps_a_quality_resource_tradeoff() {
    let mut efficient = action(1, 64_000, 100, 2_000);
    efficient.forecast = efficient.forecast.with_quality(100_000);
    let mut higher_quality = action(2, 64_000, 100, 2_000)
        .with_resources(crate::adaptive::ResourceCost::new(128_000, 128_000, 0, 1));
    higher_quality.forecast = higher_quality.forecast.with_quality(200_000);

    let frontier = ActionFrontier::prune(
        vec![efficient.clone(), higher_quality.clone()],
        EpsilonBuckets::disabled(),
    );

    assert_eq!(frontier.retained, [efficient, higher_quality]);
    assert!(frontier.pruned_ids.is_empty());
}
