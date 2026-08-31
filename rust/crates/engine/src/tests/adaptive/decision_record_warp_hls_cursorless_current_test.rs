use crate::adaptive::{
    AdaptivePlayabilityPolicy, DecisionPrivacy, DecisionRecord, FeedOffset, HlsBootstrapStage,
    HlsBootstrapState, HlsCandidateSnapshot, PlannerContext, ShadowPrices, ViewProbability,
    WarpDecisionRecordInput, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

#[test]
fn current_cursorless_hls_record_explicitly_selects_bounded_replay() {
    let mut state = snapshot(0, 80_000_000, 0, 0);
    state.request_slice_bytes = 256 * 1024;
    state.hls_candidates.push(candidate());
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let context = PlannerContext::explicitly_unavailable(&state)
        .with_segmented_storage_available_bytes(2 * 1024 * 1024);
    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &context,
    ));
    let record = DecisionRecord::capture_warp(WarpDecisionRecordInput {
        sequence: 75,
        snapshot: &state,
        decision: &decision,
        legacy_shadow_prices: ShadowPrices::default(),
        models: &[],
        privacy: &DecisionPrivacy::from_key([21; 32]),
    });
    let json = serde_json::to_value(record).expect("valid test fixture");
    let selected = &json["warp_decision"]["selected"];

    assert_eq!(selected["command"]["maximum_bytes"], 256 * 1024);
    assert!(selected["kind"].get("cursor").is_none());
    assert!(selected["command"].get("cursor").is_none());
    assert_eq!(
        json["warp_decision"]["planner_replay_capsule"]["hls_generation_policy"],
        "bounded_object_cursor"
    );
    let restored: DecisionRecord = serde_json::from_value(json).expect("valid test fixture");
    assert!(restored.replay_warp_search().is_ok());
}

fn candidate() -> HlsCandidateSnapshot {
    HlsCandidateSnapshot {
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
    }
}
