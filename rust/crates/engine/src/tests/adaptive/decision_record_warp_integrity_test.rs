use super::decision_record_warp_test_support::{decision, record, record_for};
use crate::adaptive::{
    ActionKind, AdaptivePlayabilityPolicy, DecisionPrivacy, DecisionRecord, DecisionRecordInput,
    DecisionReplayStatus, PlannerCommand, ShadowPrices,
};
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

#[test]
fn schema_two_detects_state_and_decision_tampering_before_replay_unavailable() {
    let decision = decision(
        "secret-post",
        PlannerCommand::ProbeHead {
            post: PostId::new("secret-post"),
            source: "https://origin.example/raw?token=secret".into(),
            authority: crate::adaptive::PreemptionAuthority::Transition,
        },
        ActionKind::Head,
    );
    let record = record(&decision);
    let mut value = serde_json::to_value(&record).unwrap();

    value["replay_state"]["observed_at_ms"] = serde_json::json!(123);
    let tampered: crate::adaptive::DecisionRecord = serde_json::from_value(value).unwrap();
    assert_eq!(tampered.replay(), DecisionReplayStatus::StateHashMismatch);

    let mut value = serde_json::to_value(record).unwrap();
    value["warp_decision"]["selected"]["post_id"] = serde_json::json!("changed");
    let tampered: crate::adaptive::DecisionRecord = serde_json::from_value(value).unwrap();
    assert_eq!(tampered.replay(), DecisionReplayStatus::PlanMismatch);
}

#[test]
fn schema_two_binds_the_recorded_state_to_the_authoritative_decision() {
    let decision = head_decision();
    let mut first = serde_json::to_value(record(&decision)).unwrap();
    let other_state = snapshot(2, 20_000_000, 8_000, 18);
    let second = serde_json::to_value(record_for(&decision, &other_state)).unwrap();
    first["state_hash"] = second["state_hash"].clone();
    first["replay_state"] = second["replay_state"].clone();
    let spliced: DecisionRecord = serde_json::from_value(first).unwrap();
    assert_eq!(spliced.replay(), DecisionReplayStatus::PlanMismatch);

    let mut value = serde_json::to_value(record(&decision)).unwrap();
    value["replay_plan_hash"] = serde_json::json!("unchecked");
    let tampered: DecisionRecord = serde_json::from_value(value).unwrap();
    assert_eq!(tampered.replay(), DecisionReplayStatus::PlanMismatch);
}

#[test]
fn schema_two_state_domain_differs_from_legacy_for_the_same_private_state() {
    let state = snapshot(1, 20_000_000, 8_000, 18);
    let privacy = DecisionPrivacy::from_key([5; 32]);
    let plan = AdaptivePlayabilityPolicy.plan(&state);
    let legacy = DecisionRecord::capture(DecisionRecordInput {
        sequence: 9,
        snapshot: &state,
        allocation: &plan,
        shadow_prices: ShadowPrices::default(),
        models: &[],
        privacy: &privacy,
    });
    let warp = record_for(&head_decision(), &state);
    assert_ne!(warp.state_hash, legacy.state_hash);
    assert!(warp.state_hash.starts_with("warp-v2-state:"));
}

#[test]
fn unsupported_schema_payload_pairs_never_fall_back_to_legacy_replay() {
    let decision = head_decision();
    let mut value = serde_json::to_value(record(&decision)).unwrap();
    value["schema_version"] = serde_json::json!(1);
    let record: DecisionRecord = serde_json::from_value(value).unwrap();
    assert_eq!(record.replay(), DecisionReplayStatus::UnsupportedSchema);
}

fn head_decision() -> crate::adaptive::WarpPlanningDecision {
    decision(
        "secret-post",
        PlannerCommand::ProbeHead {
            post: PostId::new("secret-post"),
            source: "https://origin.example/media".into(),
            authority: crate::adaptive::PreemptionAuthority::Transition,
        },
        ActionKind::Head,
    )
}
