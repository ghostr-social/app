use crate::adaptive::{
    AdaptivePlayabilityPolicy, DecisionPrivacy, DecisionRecord, DecisionReplayStatus, FeedOffset,
    HlsBootstrapStage, HlsBootstrapState, HlsCandidateSnapshot, PlannerContext, ResourceFeedback,
    ResourceObservation, ShadowPrices, ViewProbability, WarpDecisionRecordInput, WarpPlanner,
    WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

const SOURCE: &str = "https://private.example/root.m3u8";

#[test]
fn cursorless_hls_record_replays_historical_whole_stage_budgeting() {
    let state = hls_state();
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let context = PlannerContext::explicitly_unavailable(&state)
        .with_segmented_storage_available_bytes(512 * 1024)
        .with_feedback(storage_pressure());
    let decision = WarpPlanner::default().plan_legacy_hls_for_test(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &context,
    ));
    let record = DecisionRecord::capture_warp(WarpDecisionRecordInput {
        sequence: 72,
        snapshot: &state,
        decision: &decision,
        legacy_shadow_prices: ShadowPrices::default(),
        models: &[],
        privacy: &DecisionPrivacy::from_key([18; 32]),
    });
    let json = serde_json::to_value(record).expect("valid test fixture");
    let pruned = &json["warp_decision"]["unattributed_pre_search_pruned_actions"][0];

    assert!(json["warp_decision"]["selected"].is_null());
    assert_eq!(pruned["command"]["maximum_bytes"], 1024 * 1024);
    assert_eq!(pruned["resources"]["network_bytes"], 256 * 1024);
    assert_eq!(pruned["resources"]["storage_bytes"], 1024 * 1024);
    assert_eq!(pruned["resources"]["requests"], 1);
    assert_eq!(pruned["ready_playback_ms"], 0);
    assert!(json["warp_decision"]["prices"]["storage_micros"]
        .as_u64()
        .is_some_and(|price| price > 0));
    assert!(!serde_json::to_string(&json)
        .expect("valid test fixture")
        .contains("cursor"));
    assert!(!serde_json::to_string(&json)
        .expect("valid test fixture")
        .contains("revision"));
    assert!(json["warp_decision"]["planner_replay_capsule"]
        .get("hls_generation_policy")
        .is_none());

    let restored: DecisionRecord = serde_json::from_value(json).expect("valid test fixture");
    assert_eq!(restored.integrity_status(), DecisionReplayStatus::Verified);
    assert!(restored.replay_warp_search().is_ok());
}

fn storage_pressure() -> ResourceFeedback {
    ResourceFeedback {
        revision: 0,
        actual: ResourceObservation::new(0, 2, 0, 0),
        target: ResourceObservation::new(0, 1, 0, 0),
        price_snapshot: None,
    }
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
        state: HlsBootstrapState::Pending {
            stage: HlsBootstrapStage::RootManifest,
            source: SOURCE.into(),
        },
    });
    state
}
