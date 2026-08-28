use super::support::planned;
use crate::adaptive::{
    DecisionPrivacy, DecisionRecord, DecisionReplayStatus, ShadowPrices, WarpDecisionRecordInput,
};

#[test]
fn observed_response_promotion_round_trips_exact_replay_evidence() {
    let record = promotion_record(Some(200_000), false);
    let json = serde_json::to_value(&record).expect("valid test fixture");
    let selected = promotion(&json);

    assert_eq!(selected["command"]["command"], "promote");
    assert_eq!(selected["command"]["grant"]["maximum_bytes"], 200_000);
    assert_eq!(
        selected["resources"],
        serde_json::json!({
            "network_bytes": 200_000, "storage_bytes": 200_000,
            "cpu_ms": 0, "requests": 0,
        })
    );
    assert_eq!(
        selected["authorized_resources"],
        serde_json::json!({
            "network_bytes": 136_000, "storage_bytes": 136_000,
            "cpu_ms": 0, "requests": 0,
        })
    );
    assert_eq!(
        json["warp_decision"]["planner_replay_capsule"]["promotion_generation_policy"],
        "observed_response"
    );
    assert!(json["replay_state"]["candidates"][0]["in_flight"][0]
        .get("promotion_opportunity")
        .is_some());
    assert_round_trip(&json, &record);
}

#[test]
fn marker_free_legacy_record_replays_latent_promotion_byte_exactly() {
    let record = promotion_record(None, true);
    let encoded = serde_json::to_string(&record).expect("valid test fixture");
    let json: serde_json::Value = serde_json::from_str(&encoded).expect("valid test fixture");
    let selected = promotion(&json);

    assert_eq!(selected["command"]["command"], "promote");
    assert_eq!(selected["command"]["grant"]["maximum_bytes"], 800_000);
    assert!(json["warp_decision"]["planner_replay_capsule"]
        .get("promotion_generation_policy")
        .is_none());
    assert!(json["replay_state"]["candidates"][0]["in_flight"][0]
        .get("promotion_opportunity")
        .is_none());
    let restored: DecisionRecord = serde_json::from_str(&encoded).expect("valid test fixture");
    assert_eq!(
        serde_json::to_string(&restored).expect("valid fixture"),
        encoded
    );
    assert_verified(&restored);
}

fn promotion(json: &serde_json::Value) -> &serde_json::Value {
    json["warp_decision"]["admissible_actions"]
        .as_array()
        .expect("admissible actions")
        .iter()
        .find(|item| item["command"]["command"] == "promote")
        .expect("admissible promotion")
}

fn promotion_record(observed: Option<u64>, legacy: bool) -> DecisionRecord {
    let (state, decision) = planned(observed, legacy);
    DecisionRecord::capture_warp(WarpDecisionRecordInput {
        sequence: 91,
        snapshot: &state,
        decision: &decision,
        legacy_shadow_prices: ShadowPrices::default(),
        models: &[],
        privacy: &DecisionPrivacy::from_key([29; 32]),
    })
}

fn assert_round_trip(json: &serde_json::Value, record: &DecisionRecord) {
    let restored: DecisionRecord = serde_json::from_value(json.clone()).expect("valid fixture");
    assert_eq!(
        serde_json::to_value(&restored).expect("valid fixture"),
        *json
    );
    assert_eq!(&restored, record);
    assert_verified(&restored);
}

fn assert_verified(record: &DecisionRecord) {
    assert_eq!(record.integrity_status(), DecisionReplayStatus::Verified);
    assert_eq!(
        record.search_integrity_status(),
        DecisionReplayStatus::Verified
    );
    assert!(record.replay_warp_search().is_ok());
}
