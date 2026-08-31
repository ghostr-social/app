use crate::adaptive::{
    AdaptivePlayabilityPolicy, AllocationPlan, DecisionPrivacy, DecisionRecord,
    DecisionReplayStatus, InFlightAction, PlannerCommand, PlannerContext, PlayabilitySnapshot,
    PlayerPreparation, ReserveDegradedReason, ShadowPrices, WarpDecisionRecordInput, WarpPlanner,
    WarpPlannerConfig, WarpPlannerInput, WarpPlanningDecision,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::{ActionId, ByteRange, PostId};

#[test]
fn degraded_safety_fills_missing_reserve_before_ready_video_coverage() {
    let (input, base) = scenario(3_000);
    let decision = plan(&input, &base, WarpPlannerConfig::default());
    assert!(decision.reserve.degraded);
    assert_eq!(
        decision.reserve.degraded_reason,
        Some(ReserveDegradedReason::NoFeasibleRescue)
    );
    assert_fills_fifth(&decision);
    assert_replays(&input, &decision);
}
#[test]
fn chance_feasible_safety_executes_the_protected_reserve_path_first() {
    let (input, base) = scenario(20_000);
    let config = WarpPlannerConfig::default().with_rescue_thresholds(0, 0);
    let decision = plan(&input, &base, config);
    assert!(!decision.reserve.degraded);
    assert_fills_fifth(&decision);
}
fn scenario(commitment_ms: u64) -> (PlayabilitySnapshot, AllocationPlan) {
    let mut input = snapshot(6, 2_500_000, 20_000, 120);
    input.commitment_ms = commitment_ms;
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
    input.candidates[1].in_flight.push(InFlightAction::range(
        ActionId::new(7),
        ByteRange::new(250_000, 500_000),
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
    (input, base)
}
fn plan(
    input: &crate::adaptive::PlayabilitySnapshot,
    base: &crate::adaptive::AllocationPlan,
    config: WarpPlannerConfig,
) -> WarpPlanningDecision {
    let context = PlannerContext::explicitly_unavailable(input);
    WarpPlanner::new(config).plan(WarpPlannerInput::new(
        input,
        base,
        &OriginModel::default(),
        &context,
    ))
}
fn assert_fills_fifth(decision: &WarpPlanningDecision) {
    let selected = decision.selected.as_ref().expect("fifth reserve transfer");
    let reserve = &decision.reserve;
    assert_eq!(selected.node.post, PostId::new("p5"));
    assert!(reserve.degraded || reserve.protected_action_ids.contains(&selected.node.id));
    assert!(matches!(selected.command, PlannerCommand::Transfer(_)));
}
fn assert_replays(input: &crate::adaptive::PlayabilitySnapshot, decision: &WarpPlanningDecision) {
    let mut decision = decision.clone();
    decision.planner_replay = None;
    let record = DecisionRecord::capture_warp(WarpDecisionRecordInput {
        sequence: 1,
        snapshot: input,
        decision: &decision,
        legacy_shadow_prices: ShadowPrices::default(),
        models: &[],
        privacy: &DecisionPrivacy::from_key([7; 32]),
    });
    let json = serde_json::to_string(&record).expect("search replay record");
    assert!(json.contains("reserve_progress_action_ids"));
    let restored: DecisionRecord = serde_json::from_str(&json).expect("search replay record");
    assert_eq!(
        restored.search_integrity_status(),
        DecisionReplayStatus::Verified
    );
    assert!(restored.replay_warp_search().is_ok());
}
