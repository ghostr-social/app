use crate::adaptive::{AdaptivePlayabilityPolicy, ControlMode, FeedOffset};
use crate::tests::adaptive_support::snapshot;
use crate::PostId;
use std::collections::HashSet;

#[test]
fn rapid_swipes_and_slow_recovery_prepare_several_distinct_videos() {
    let input = snapshot(8, 2_000_000, 20_000, 60);

    let plan = AdaptivePlayabilityPolicy.plan(&input);
    let prepared: HashSet<_> = plan
        .allocations
        .iter()
        .filter(|work| work.post != input.playback.current)
        .map(|work| work.post.clone())
        .collect();

    assert!(plan.ready_reserve.target >= 3);
    assert!(
        plan.ready_reserve.protected >= 3,
        "reserve: {:?}",
        plan.ready_reserve
    );
    assert!(prepared.len() >= 3);
    assert_eq!(plan.mode, ControlMode::Emergency);
}

#[test]
fn patient_navigation_and_fast_recovery_keep_a_lean_reserve() {
    let plan = AdaptivePlayabilityPolicy.plan(&snapshot(8, 20_000_000, 20_000, 0));

    assert_eq!(plan.ready_reserve.target, 1);
}

#[test]
fn playable_prefixes_count_as_probability_weighted_ready_coverage() {
    let mut input = snapshot(6, 1_000_000, 20_000, 0);
    for candidate in &mut input.candidates[1..=2] {
        candidate.present.push(candidate.playable_ranges[0].bytes);
    }

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert_eq!(plan.ready_reserve.ready, 2);
    assert!(plan.ready_reserve.ready_coverage_ms > 3_000);
    assert_eq!(plan.mode, ControlMode::Normal);
}

#[test]
fn a_swipe_replenishes_the_consumed_reserve_at_the_far_edge() {
    let mut input = snapshot(7, 1_000_000, 20_000, 60);
    input.playback.current = PostId::new("p1");
    for (index, candidate) in input.candidates.iter_mut().enumerate() {
        candidate.feed_offset = FeedOffset::new(index as i32 - 1);
    }
    for candidate in &mut input.candidates[2..=4] {
        candidate.present.push(candidate.playable_ranges[0].bytes);
    }

    let plan = AdaptivePlayabilityPolicy.plan(&input);
    let refilled: HashSet<_> = plan
        .allocations
        .iter()
        .map(|work| work.post.as_str())
        .collect();

    assert_eq!(plan.ready_reserve.ready, 3);
    assert_eq!(plan.ready_reserve.protected, plan.ready_reserve.target);
    assert!(refilled.contains("p5"));
    assert!(refilled.contains("p6"));
}
