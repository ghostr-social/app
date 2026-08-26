use crate::adaptive::{
    AdaptivePlayabilityPolicy, DecisionOutcome, DecisionPrivacy, DecisionRecord,
    DecisionReplayStatus, FeedOffset, HlsBootstrapStage, HlsBootstrapState, HlsCandidateSnapshot,
    HlsObjectCursor, HlsTransport, PlannerContext, ShadowPrices, ViewProbability,
    WarpDecisionRecordInput, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::{ActionId, PostId};

const SOURCE: &str = "https://private.example/root.m3u8";

#[test]
fn hls_bootstrap_commitment_is_private_typed_and_fully_replayable() {
    let record = hls_record();
    let json = serde_json::to_value(&record).expect("valid test fixture");
    let selected = &json["warp_decision"]["selected"];
    let cursor = serde_json::to_value(resume_cursor()).expect("valid test fixture");

    assert_eq!(selected["kind"]["kind"], "hls_bootstrap");
    assert_eq!(selected["kind"]["stage"], "root_manifest");
    assert_eq!(selected["command"]["command"], "fetch_hls_bootstrap");
    assert_eq!(selected["resources"]["network_bytes"], 44 * 1024);
    assert_eq!(selected["command"]["maximum_bytes"], 44 * 1024);
    assert_eq!(json["replay_state"]["hls_candidates"][0]["cursor"], cursor);
    assert_eq!(selected["kind"]["cursor"], cursor);
    assert_eq!(selected["command"]["cursor"], cursor);
    assert_eq!(
        json["warp_decision"]["planner_replay_capsule"]["hls_generation_policy"],
        "bounded_object_cursor"
    );
    assert_ne!(selected["command"]["source_id"], SOURCE);
    assert!(!serde_json::to_string(&record)
        .expect("valid test fixture")
        .contains(SOURCE));
    let restored: DecisionRecord =
        serde_json::from_value(json.clone()).expect("valid test fixture");
    assert_eq!(restored.integrity_status(), DecisionReplayStatus::Verified);
    assert!(restored.replay_warp_search().is_ok());
    for location in ["kind", "command"] {
        let mut tampered = json.clone();
        tampered["warp_decision"]["selected"][location]["cursor"]["attempt"] = serde_json::json!(8);
        let tampered: DecisionRecord =
            serde_json::from_value(tampered).expect("valid test fixture");
        assert_eq!(
            tampered.integrity_status(),
            DecisionReplayStatus::PlanMismatch
        );
    }
}

#[test]
fn hls_terminal_failure_class_is_sealed_against_replay_mutation() {
    let mut record = hls_record();
    assert!(record.bind_action(ActionId::new(71)));
    assert!(record.resolve(DecisionOutcome::Failed {
        class: "warp_hls_http_5xx".to_owned(),
        elapsed_ms: 8,
    }));
    assert_eq!(record.integrity_status(), DecisionReplayStatus::Verified);

    let mut tampered = serde_json::to_value(record).expect("valid test fixture");
    tampered["eventual_outcome"]["class"] = serde_json::json!("warp_hls_policy");
    let tampered: DecisionRecord = serde_json::from_value(tampered).expect("valid test fixture");
    assert_eq!(
        tampered.integrity_status(),
        DecisionReplayStatus::PlanMismatch
    );
}

fn hls_record() -> DecisionRecord {
    let mut state = snapshot(0, 80_000_000, 0, 0);
    state.hls_candidates.push(HlsCandidateSnapshot {
        post: PostId::new("p0"),
        feed_offset: FeedOffset::new(0),
        view_probability: ViewProbability::new(1.0).expect("valid test fixture"),
        startup_value_ms: 2_000,
        cursor: resume_cursor(),
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

fn resume_cursor() -> HlsObjectCursor {
    HlsObjectCursor::new(7, 256 * 1024, Some(300 * 1024), HlsTransport::ResumeRange)
}
