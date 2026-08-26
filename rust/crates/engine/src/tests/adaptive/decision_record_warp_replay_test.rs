use crate::adaptive::{
    AdaptivePlayabilityPolicy, BeamConfig, DecisionPrivacy, DecisionRecord, DecisionReplayStatus,
    PlannerContext, ShadowPrices, WarpDecisionRecordInput, WarpPlanner, WarpPlannerConfig,
    WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn a_complete_production_warp_trace_replays_as_a_typed_verified_result() {
    let state = snapshot(1, 20_000_000, 8_000, 18);
    let (captured, decision) = capture(&state);
    let exported: DecisionRecord =
        serde_json::from_str(&serde_json::to_string(&captured).expect("valid test fixture"))
            .expect("serialized production decision record");
    let replay = exported.replay_warp().expect("complete WARP replay");
    let fresh = exported
        .replay_warp_search()
        .expect("deterministic WARP search replay");

    assert_eq!(exported.integrity_status(), DecisionReplayStatus::Verified);
    assert_eq!(fresh, replay);
    assert_eq!(replay.sequence(), 41);
    assert_eq!(
        replay.decision(),
        exported.warp_decision.as_ref().expect("valid test fixture")
    );
    assert!(decision.selected.is_some(), "fixture requires a commitment");
    assert_eq!(
        replay.selected(),
        exported
            .warp_decision
            .as_ref()
            .expect("valid test fixture")
            .selected
            .as_ref()
    );
    assert_eq!(
        replay.search(),
        &exported
            .warp_decision
            .as_ref()
            .expect("valid test fixture")
            .search
    );
    assert_eq!(replay.common_random_seed(), decision.common_random_seed);
    assert_eq!(
        replay.prices().network_micros,
        decision.prices.network_micros
    );
    assert_eq!(
        replay.reserve(),
        exported
            .warp_decision
            .as_ref()
            .expect("valid test fixture")
            .reserve
            .clone()
    );
    assert_eq!(replay.integrity().state_hash(), exported.state_hash);
    assert!(replay
        .integrity()
        .decision_hash()
        .starts_with("warp-v2-decision:"));
}

#[test]
fn capability_blocked_warp_state_uses_a_new_replay_schema() {
    let mut state = snapshot(1, 20_000_000, 8_000, 18);
    state.candidates[0].direct_playback_blocked = true;
    let (record, _) = capture(&state);

    assert_eq!(record.schema_version, 4);
    assert_eq!(record.integrity_status(), DecisionReplayStatus::Verified);
}

fn capture(
    state: &crate::adaptive::PlayabilitySnapshot,
) -> (DecisionRecord, crate::adaptive::WarpPlanningDecision) {
    let base = AdaptivePlayabilityPolicy.plan(state);
    let context = PlannerContext::explicitly_unavailable(state);
    let config = WarpPlannerConfig {
        beam: BeamConfig::new(1, 4, 16, u64::MAX),
        ..WarpPlannerConfig::default()
    };
    let decision = WarpPlanner::new(config).plan(WarpPlannerInput::new(
        state,
        &base,
        &OriginModel::default(),
        &context,
    ));
    let captured = DecisionRecord::capture_warp(WarpDecisionRecordInput {
        sequence: 41,
        snapshot: state,
        decision: &decision,
        legacy_shadow_prices: ShadowPrices::default(),
        models: &[],
        privacy: &DecisionPrivacy::from_key([11; 32]),
    });
    (captured, decision)
}
