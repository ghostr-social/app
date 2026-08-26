use super::support::transfer_record;
use crate::adaptive::{DecisionRecord, DecisionReplayStatus};

#[test]
fn legacy_bound_schema_two_shape_round_trips_without_hash_drift() {
    let mut record = transfer_record();
    record.emulate_legacy_warp_v2();
    let hash = record
        .replay_warp()
        .expect("valid test fixture")
        .integrity()
        .decision_hash()
        .to_owned();
    let mut value = serde_json::to_value(record).expect("valid test fixture");
    value["chosen_action_id"] = serde_json::json!(44);
    let json = serde_json::to_string(&value).expect("valid test fixture");

    assert!(!json.contains("executed_request"));
    let restored: DecisionRecord = serde_json::from_str(&json).expect("valid test fixture");
    assert_eq!(
        serde_json::to_value(&restored).expect("valid test fixture"),
        value
    );
    assert_eq!(restored.integrity_status(), DecisionReplayStatus::Verified);
    assert_eq!(
        restored
            .replay_warp()
            .expect("valid test fixture")
            .integrity()
            .decision_hash(),
        hash
    );
}
