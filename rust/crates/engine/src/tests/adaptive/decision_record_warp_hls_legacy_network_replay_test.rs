use crate::adaptive::{
    AdaptivePlayabilityPolicy, DecisionPrivacy, DecisionRecord, FeedOffset, HlsBootstrapStage,
    HlsBootstrapState, HlsCandidateSnapshot, PlannerContext, ShadowPrices, ViewProbability,
    WarpDecisionRecordInput, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

#[test]
fn cursorless_hls_record_replays_historical_network_envelope() {
    let state = hls_state();
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let mut context = PlannerContext::explicitly_unavailable(&state)
        .with_segmented_storage_available_bytes(2 * 1024 * 1024);
    context.limits.network_burst_bytes = 512 * 1024;
    context.limits.network_rate_bytes_per_second = 0;
    let decision = WarpPlanner::default().plan_legacy_hls_for_test(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &context,
    ));
    let record = DecisionRecord::capture_warp(WarpDecisionRecordInput {
        sequence: 73,
        snapshot: &state,
        decision: &decision,
        legacy_shadow_prices: ShadowPrices::default(),
        models: &[],
        privacy: &DecisionPrivacy::from_key([19; 32]),
    });
    let json = serde_json::to_value(record).expect("valid test fixture");
    let pruned = &json["warp_decision"]["unattributed_pre_search_pruned_actions"][0];

    assert!(json["warp_decision"]["selected"].is_null());
    assert_eq!(pruned["resources"]["network_bytes"], 256 * 1024);
    assert_eq!(pruned["command"]["maximum_bytes"], 1024 * 1024);
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
            stage: HlsBootstrapStage::RootManifest,
            source: "https://private.example/root.m3u8".into(),
        },
    });
    state
}
