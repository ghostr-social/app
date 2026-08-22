use crate::adaptive::{AdaptivePlayabilityPolicy, PlayerPreparation, ReserveCandidateState};
use crate::tests::adaptive_support::snapshot;

#[test]
fn an_arbitrary_prefix_without_a_startup_footprint_is_not_protected() {
    let mut input = snapshot(2, 20_000_000, 20_000, 2);
    input.candidates[0].present = input.candidates[0]
        .playable_ranges
        .iter()
        .take(5)
        .map(|range| range.bytes)
        .collect();
    input.candidates[1].startup = None;
    input.candidates[1].player_preparation = PlayerPreparation::Unverified;

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert_eq!(plan.ready_reserve.protected, 0);
    assert!(matches!(
        plan.ready_reserve.candidates[0].state,
        ReserveCandidateState::Preparing { .. }
    ));
}
