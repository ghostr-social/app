use super::support::transfer_record;
use crate::adaptive::{DecisionRecord, DecisionReplayStatus};

#[test]
fn legacy_bound_schema_two_shape_round_trips_without_hash_drift() {
    let record = transfer_record();
    let hash = record
        .replay_warp()
        .unwrap()
        .integrity()
        .decision_hash()
        .to_owned();
    let mut value = serde_json::to_value(record).unwrap();
    value["chosen_action_id"] = serde_json::json!(44);
    let json = serde_json::to_string(&value).unwrap();

    assert!(!json.contains("executed_request"));
    let restored: DecisionRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(serde_json::to_value(&restored).unwrap(), value);
    assert_eq!(restored.replay(), DecisionReplayStatus::Verified);
    assert_eq!(
        restored.replay_warp().unwrap().integrity().decision_hash(),
        hash
    );
}
