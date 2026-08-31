use crate::adaptive::{
    AdaptivePlayabilityPolicy, DecisionPrivacy, DecisionRecord, DecisionReplayStatus,
    PlannerContext, ShadowPrices, WarpDecisionRecordInput, WarpPlanner, WarpPlannerConfig,
    WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::{ByteRange, PostId};

#[test]
fn marker_free_capsule_replays_historical_coverage_first_selection() {
    let mut state = snapshot(6, 2_500_000, 20_000, 120);
    state.commitment_ms = 3_000;
    state.candidates[0].present = vec![ByteRange::new(0, 3_750_000)];
    for candidate in &mut state.candidates[1..=4] {
        candidate.present = candidate
            .startup
            .as_ref()
            .expect("reserve startup")
            .ranges()
            .to_vec();
    }
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let context = PlannerContext::explicitly_unavailable(&state);
    let config = WarpPlannerConfig::default().with_legacy_reserve_progress_for_test();
    let decision = WarpPlanner::new(config).plan(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &context,
    ));
    assert_ne!(
        decision
            .selected
            .as_ref()
            .expect("selected legacy action")
            .node
            .post,
        PostId::new("p5")
    );
    let record = DecisionRecord::capture_warp(WarpDecisionRecordInput {
        sequence: 1,
        snapshot: &state,
        decision: &decision,
        legacy_shadow_prices: ShadowPrices::default(),
        models: &[],
        privacy: &DecisionPrivacy::from_key([8; 32]),
    });
    let json = serde_json::to_string(&record).expect("legacy record");
    assert!(!json.contains("reserve_progress_policy"));
    let restored: DecisionRecord = serde_json::from_str(&json).expect("legacy record");
    assert_eq!(restored.integrity_status(), DecisionReplayStatus::Verified);
    assert!(restored.replay_warp_search().is_ok());
}
