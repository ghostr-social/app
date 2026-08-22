use crate::adaptive::{
    AdaptivePlayabilityPolicy, DecisionOutcome, DecisionPrivacy, DecisionRecord,
    DecisionReplayStatus, FeedOffset, HlsBootstrapStage, HlsBootstrapState, HlsCandidateSnapshot,
    PlannerContext, ShadowPrices, ViewProbability, WarpDecisionRecordInput, WarpPlanner,
    WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::{ActionId, PostId};

const SOURCE: &str = "https://private.example/root.m3u8";

#[test]
fn hls_bootstrap_commitment_is_private_typed_and_fully_replayable() {
    let record = hls_record();
    let json = serde_json::to_value(&record).unwrap();
    let selected = &json["warp_decision"]["selected"];

    assert_eq!(selected["kind"]["kind"], "hls_bootstrap");
    assert_eq!(selected["kind"]["stage"], "root_manifest");
    assert_eq!(selected["command"]["command"], "fetch_hls_bootstrap");
    assert_eq!(selected["resources"]["network_bytes"], 256 * 1024);
    assert_eq!(selected["command"]["maximum_bytes"], 1024 * 1024);
    assert_ne!(selected["command"]["source_id"], SOURCE);
    assert!(!serde_json::to_string(&record).unwrap().contains(SOURCE));
    assert_eq!(record.replay(), DecisionReplayStatus::Verified);
    assert!(record.replay_warp_search().is_ok());
}

#[test]
fn hls_terminal_failure_class_is_sealed_against_replay_mutation() {
    let mut record = hls_record();
    assert!(record.bind_action(ActionId::new(71)));
    assert!(record.resolve(DecisionOutcome::Failed {
        class: "warp_hls_http_5xx".to_owned(),
        elapsed_ms: 8,
    }));
    assert_eq!(record.replay(), DecisionReplayStatus::Verified);

    let mut tampered = serde_json::to_value(record).unwrap();
    tampered["eventual_outcome"]["class"] = serde_json::json!("warp_hls_policy");
    let tampered: DecisionRecord = serde_json::from_value(tampered).unwrap();
    assert_eq!(tampered.replay(), DecisionReplayStatus::PlanMismatch);
}

fn hls_record() -> DecisionRecord {
    let mut state = snapshot(0, 80_000_000, 0, 0);
    state.hls_candidates.push(HlsCandidateSnapshot {
        post: PostId::new("p0"),
        feed_offset: FeedOffset::new(0),
        view_probability: ViewProbability::new(1.0).unwrap(),
        startup_value_ms: 2_000,
        state: HlsBootstrapState::Pending {
            stage: HlsBootstrapStage::RootManifest,
            source: SOURCE.into(),
        },
    });
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let context = PlannerContext::explicitly_unavailable(&state)
        .with_segmented_storage_available_bytes(1024 * 1024);
    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &context,
    ));
    DecisionRecord::capture_warp(WarpDecisionRecordInput {
        sequence: 71,
        snapshot: &state,
        decision: &decision,
        legacy_shadow_prices: ShadowPrices::default(),
        models: &[],
        privacy: &DecisionPrivacy::from_key([17; 32]),
    })
}
