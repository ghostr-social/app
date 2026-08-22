use super::support::{planned, record};
use crate::adaptive::{DecisionRecord, DecisionReplayStatus};

#[test]
fn capsule_projects_every_planning_identifier_and_detects_tampering() {
    let (state, decision) = planned();
    let captured = record(&state, &decision);
    let json = serde_json::to_string(&captured).unwrap();
    for secret in ["origin.example", "active.example", "\"p0\"", "\"p1\""] {
        assert!(!json.contains(secret), "planner capsule leaked {secret}");
    }
    let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
    value["warp_decision"]["planner_replay_capsule"]["config"]["safety_rescue_bps"] = 9_499.into();
    let tampered: DecisionRecord = serde_json::from_value(value).unwrap();

    assert_eq!(tampered.replay(), DecisionReplayStatus::PlanMismatch);
    assert_eq!(
        tampered.replay_warp_search(),
        Err(DecisionReplayStatus::PlanMismatch)
    );
}
