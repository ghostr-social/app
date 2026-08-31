use crate::adaptive::{
    AdaptivePlayabilityPolicy, DecisionPrivacy, DecisionRecord, FeedOffset, HlsBootstrapStage,
    HlsBootstrapState, HlsCandidateSnapshot, PlannerContext, ShadowPrices, ViewProbability,
    WarpDecisionRecordInput, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

#[test]
fn cursorless_first_segment_replays_historical_ready_credit() {
    let state = hls_state();
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let context = PlannerContext::explicitly_unavailable(&state)
        .with_segmented_storage_available_bytes(16 * 1024 * 1024);
    let decision = WarpPlanner::default().plan_legacy_hls_for_test(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &context,
    ));
    let record = DecisionRecord::capture_warp(WarpDecisionRecordInput {
        sequence: 74,
        snapshot: &state,
        decision: &decision,
        legacy_shadow_prices: ShadowPrices::default(),
        models: &[],
        privacy: &DecisionPrivacy::from_key([20; 32]),
    });
    let json = serde_json::to_value(record).expect("valid test fixture");
    let selected = &json["warp_decision"]["selected"];

    assert_eq!(selected["ready_playback_ms"], 2_000);
    assert_eq!(selected["command"]["maximum_bytes"], 8 * 1024 * 1024);
    assert!(!serde_json::to_string(&json)
        .expect("valid test fixture")
        .contains("cursor"));
    let restored: DecisionRecord = serde_json::from_value(json).expect("valid test fixture");
    assert!(restored.replay_warp_search().is_ok());
}

fn hls_state() -> crate::adaptive::PlayabilitySnapshot {
    let mut state = snapshot(0, 80_000_000, 0, 0);
    state.request_slice_bytes = 256 * 1024;
    state.hls_candidates.push(HlsCandidateSnapshot {
        post: PostId::new("p0"),
        feed_offset: FeedOffset::new(0),
        view_probability: ViewProbability::new(1.0).expect("valid test fixture"),
        startup_value_ms: 2_000,
        cursor: Default::default(),
        player_preparation: Default::default(),
        state: HlsBootstrapState::Pending {
            stage: HlsBootstrapStage::FirstSegment,
            source: "https://private.example/first.m4s".into(),
        },
    });
    state
}
