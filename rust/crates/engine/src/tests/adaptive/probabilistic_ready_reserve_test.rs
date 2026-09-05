use crate::adaptive::{AdaptivePlayabilityPolicy, ControlMode, FeedOffset};
use crate::tests::adaptive_support::snapshot;
use crate::PostId;
use std::collections::HashSet;

#[test]
fn rapid_swipes_keep_the_initial_reserve_within_two_distinct_videos() {
    let input = snapshot(8, 2_000_000, 20_000, 60);

    let plan = AdaptivePlayabilityPolicy.plan(&input);
    let prepared: HashSet<_> = plan
        .allocations
        .iter()
        .filter(|work| work.post != input.playback.current)
        .map(|work| work.post.clone())
        .collect();

    assert_eq!(plan.ready_reserve.target, 2);
    assert!(
        plan.ready_reserve.protected == 2,
        "reserve: {:?}",
        plan.ready_reserve
    );
    assert!(prepared.len() <= 2);
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
    assert!(plan.ready_reserve.ready_coverage_ms > 0);
    assert!(plan.ready_reserve.ready_coverage_ms <= 4_000);
    assert_eq!(plan.mode, ControlMode::Normal);
}

#[test]
fn already_retained_nearby_items_do_not_expand_the_reserve_after_a_swipe() {
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

    assert_eq!(plan.ready_reserve.ready, 2);
    assert!(plan.ready_reserve.protected >= plan.ready_reserve.target);
    assert!(!refilled.contains("p5"));
    assert!(!refilled.contains("p6"));
}
