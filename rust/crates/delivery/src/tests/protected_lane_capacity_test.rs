use super::support::planned_transfer;
use crate::manager::concurrency::{planned_capacity, PlannedCapacity};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::PostId;
use std::collections::HashSet;

#[test]
fn playback_critical_work_opens_one_capped_transition_lane() {
    let transfers = [
        planned_transfer("current", "same", PreemptionAuthority::PlaybackCritical),
        planned_transfer("next", "same", PreemptionAuthority::Transition),
    ];

    assert_eq!(
        planned_capacity(1, 3, &transfers, &retained(&[])),
        PlannedCapacity {
            total: 2,
            foreground_goal: 1,
        }
    );
}

#[test]
fn transition_lane_never_exceeds_the_configured_ceiling() {
    let transfers = [
        planned_transfer("current", "same", PreemptionAuthority::PlaybackCritical),
        planned_transfer("next", "same", PreemptionAuthority::Transition),
    ];

    assert_eq!(planned_capacity(1, 1, &transfers, &retained(&[])).total, 1);
}

#[test]
fn a_retained_transition_opens_the_capped_lane() {
    let transfers = [planned_transfer(
        "current",
        "same",
        PreemptionAuthority::PlaybackCritical,
    )];
    assert_eq!(
        planned_capacity(1, 3, &transfers, &retained(&["next"])).total,
        2
    );
}

#[test]
fn a_retained_current_keeps_one_lane_open_for_the_next_transition() {
    let transfers = [planned_transfer(
        "next",
        "same",
        PreemptionAuthority::Transition,
    )];
    assert_eq!(
        planned_capacity(1, 3, &transfers, &retained(&["current"])).total,
        2
    );
}

#[test]
fn two_paid_transfers_do_not_block_the_immediate_next_transition() {
    let transfers = [planned_transfer(
        "next",
        "same",
        PreemptionAuthority::Transition,
    )];
    let paid = retained(&["current", "previous"]);
    assert_eq!(planned_capacity(1, 3, &transfers, &paid).total, 3);
}

#[test]
fn foreground_goal_is_bounded_by_the_adaptive_base_and_planned_work() {
    assert_plan(2, &[PreemptionAuthority::PlaybackCritical; 3], 2, 2);
    assert_plan(2, &[PreemptionAuthority::PlaybackCritical], 2, 1);
    assert_plan(1, &[PreemptionAuthority::PlaybackCritical; 2], 2, 1);
}

fn assert_plan(base: usize, foreground: &[PreemptionAuthority], total: usize, goal: usize) {
    let mut transfers: Vec<_> = foreground
        .iter()
        .map(|authority| planned_transfer("current", "same", *authority))
        .collect();
    transfers.push(planned_transfer(
        "next",
        "same",
        PreemptionAuthority::Transition,
    ));
    assert_eq!(
        planned_capacity(base, 3, &transfers, &retained(&[])),
        PlannedCapacity {
            total,
            foreground_goal: goal,
        }
    );
}

fn retained(posts: &[&str]) -> HashSet<PostId> {
    posts.iter().map(|post| PostId::new(*post)).collect()
}
