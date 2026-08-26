use super::node;
use crate::adaptive::{ActionKind, BeamConfig, HardBudget, ResourceCost, WarpSearch};
use crate::ByteRange;

#[test]
fn prune_audit_discloses_events_omitted_after_the_bounded_sample() {
    let nodes: Vec<_> = (1..=65)
        .map(|id| {
            node(
                id,
                ActionKind::FetchWhole {
                    maximum_bytes: u64::from(id),
                },
                1,
                &[],
            )
        })
        .collect();
    let budget = HardBudget::new(ResourceCost::default(), 0);
    let decision =
        WarpSearch::new(BeamConfig::new(1, 1, 128, u64::MAX)).choose_first(&nodes, budget);

    assert_eq!(decision.pruned_plans.len(), 64);
    assert_eq!(decision.pruned_plan_events_total, 65);
    assert!(decision.pruned_plan_sample_truncated);
}

#[test]
fn chosen_plan_preserves_an_earlier_best_after_the_final_beam_changes() {
    let whole = node(
        1,
        ActionKind::FetchWhole {
            maximum_bytes: 300_000,
        },
        100,
        &[],
    );
    let prefix = node(2, ActionKind::Prefix(ByteRange::new(0, 64_000)), 60, &[]);
    let tail = node(
        3,
        ActionKind::Tail(ByteRange::new(64_000, 128_000)),
        20,
        &[2],
    );
    let decision = WarpSearch::new(BeamConfig::new(2, 3, 128, u64::MAX))
        .choose_first(&[whole.clone(), prefix, tail], HardBudget::unlimited());

    assert_eq!(decision.action, Some(whole));
    assert_eq!(
        decision.chosen_plan.expect("valid test fixture").action_ids,
        vec![1]
    );
    assert!(decision
        .retained_plans
        .iter()
        .any(|plan| plan.action_ids == vec![2, 3]));
}
