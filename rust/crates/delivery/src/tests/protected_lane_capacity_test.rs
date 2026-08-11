use super::support::planned_transfer;
use crate::manager::concurrency::{effective_capacity, planned_capacity, PlannedCapacity};
use ghostr_engine::tiers::Tier;

#[test]
fn one_extra_slot_requires_both_foreground_and_protected_work() {
    let cases = [
        (1, 3, true, true, 2),
        (1, 1, true, true, 1),
        (1, 3, false, true, 1),
        (1, 3, true, false, 1),
        (2, 3, true, true, 3),
    ];

    for (base, ceiling, foreground, protected, expected) in cases {
        assert_eq!(
            effective_capacity(base, ceiling, foreground, protected),
            expected
        );
    }
}

#[test]
fn foreground_goal_is_bounded_by_the_adaptive_base_and_planned_work() {
    assert_plan(2, &[Tier::T0PlaybackEmergency; 3], 3, 2);
    assert_plan(2, &[Tier::T0PlaybackEmergency], 3, 1);
    assert_plan(1, &[Tier::T1CurrentTail; 2], 2, 1);
}

fn assert_plan(base: usize, foreground: &[Tier], total: usize, goal: usize) {
    let mut transfers: Vec<_> = foreground
        .iter()
        .enumerate()
        .map(|(index, tier)| planned_transfer(&format!("current-{index}"), "same", *tier))
        .collect();
    transfers.push(planned_transfer("next", "same", Tier::T2Startability));
    assert_eq!(
        planned_capacity(base, 3, &transfers),
        PlannedCapacity {
            total,
            foreground_goal: goal,
        }
    );
}
