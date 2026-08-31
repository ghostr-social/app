use crate::adaptive::{
    AdaptivePlayabilityPolicy, InFlightAction, PlannerContext, PlayerPreparation,
    ReserveCandidateState, WarpPlanner, WarpPlannerConfig, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::{ActionId, ByteRange};

#[test]
fn inflight_fifth_reserve_does_not_launch_an_unrelated_continuation() {
    let mut input = snapshot(6, 2_500_000, 20_000, 120);
    input.commitment_ms = 20_000;
    input.candidates[0].present = vec![ByteRange::new(0, 3_750_000)];
    for candidate in &mut input.candidates[1..=4] {
        candidate.present = candidate
            .startup
            .as_ref()
            .expect("reserve startup")
            .ranges()
            .to_vec();
    }
    input.candidates[5].player_preparation = PlayerPreparation::Unverified;
    input.candidates[5].in_flight.push(InFlightAction::range(
        ActionId::new(9),
        ByteRange::new(0, 250_000),
        "https://origin.example/media",
        13_000,
        true,
    ));
    let base = AdaptivePlayabilityPolicy.plan(&input);
    assert_eq!(
        (
            base.ready_reserve.target,
            base.ready_reserve.ordered_ready()
        ),
        (5, 4)
    );
    assert!(matches!(
        base.ready_reserve.candidates[4].state,
        ReserveCandidateState::InFlight
    ));

    let context = PlannerContext::explicitly_unavailable(&input);
    let config = WarpPlannerConfig::default().with_rescue_thresholds(0, 0);
    let decision = WarpPlanner::new(config).plan(WarpPlannerInput::new(
        &input,
        &base,
        &OriginModel::default(),
        &context,
    ));
    let replay = decision.search_replay.expect("search replay evidence");

    assert!(decision.reserve.degraded);
    assert!(decision.reserve.protected_action_ids.is_empty());
    assert!(replay.reserve_progress_action_ids().is_empty());
    assert_ne!(
        decision.selected.expect("useful fallback").node.post,
        input.candidates[5].post
    );
}
