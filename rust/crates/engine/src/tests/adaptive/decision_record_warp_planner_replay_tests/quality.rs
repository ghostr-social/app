use super::support::{planned, record};
use crate::adaptive::{DecisionRecord, DecisionReplayStatus};

#[test]
fn captured_quality_and_post_identity_are_integrity_bound() {
    for field in ["quality_gain_micros", "post_id", "origin_admission_intent"] {
        let (state, decision) = planned();
        let captured = record(&state, &decision);
        let mut value = serde_json::to_value(captured).expect("valid test fixture");
        let action = &mut value["warp_decision"]["search_replay_input"]["actions"][0];
        match field {
            "quality_gain_micros" => action["forecast"][field] = 77.into(),
            "post_id" => action[field] = "privacy-safe-different-post".into(),
            "origin_admission_intent" => toggle_intent(action),
            _ => unreachable!(),
        }
        let tampered: DecisionRecord = serde_json::from_value(value).expect("valid test fixture");
        assert_eq!(
            tampered.integrity_status(),
            DecisionReplayStatus::PlanMismatch
        );
    }
}

fn toggle_intent(action: &mut serde_json::Value) {
    let optional = action["origin_admission_intent"] == "optional_exploration";
    action["origin_admission_intent"] = if optional {
        "delivery".into()
    } else {
        "optional_exploration".into()
    };
}
