use crate::adaptive::{
    AdaptivePlayabilityPolicy, ControlMode, PlayerPreparation, ReserveCandidateState,
};
use crate::tests::adaptive_support::snapshot;

#[test]
fn a_ready_item_beyond_a_gap_does_not_satisfy_the_forward_reserve() {
    let mut input = snapshot(6, 3_000_000, 20_000, 60);
    for candidate in &mut input.candidates[1..=4] {
        let startup = candidate.startup.as_ref().expect("fixture").ranges()[0];
        candidate.present = vec![startup];
    }
    input.candidates[1].player_preparation = PlayerPreparation::Unverified;

    let plan = AdaptivePlayabilityPolicy.plan(&input);

    assert_eq!(plan.ready_reserve.target, 1);
    assert_eq!(plan.ready_reserve.ready, 1);
    assert_eq!(plan.ready_reserve.ordered_ready(), 0);
    assert!(plan.ready_reserve.ready_coverage_ms > 0);
    assert!(matches!(
        plan.ready_reserve.candidates[0].state,
        ReserveCandidateState::Structural { .. }
    ));
    assert!(!plan.ready_reserve.ordered_target_satisfied());
    assert_eq!(plan.mode, ControlMode::Emergency);
}
