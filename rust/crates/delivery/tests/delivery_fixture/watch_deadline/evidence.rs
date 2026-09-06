use ghostr_delivery::delivery_events::{DeliveryHandle, PlanEvidence};

pub(super) fn deadline_at(handle: &DeliveryHandle, plan: &PlanEvidence, offset: i32) -> u64 {
    let encoded = handle.decision_history_json().expect("decision evidence");
    let json: serde_json::Value = serde_json::from_str(&encoded).expect("decision JSON");
    let sequence = plan.decision_sequence.expect("causal plan");
    let decision = json["decisions"]["records"]
        .as_array()
        .expect("decisions")
        .iter()
        .find(|record| record["sequence"] == sequence)
        .expect("matching decision");
    let post = post_at(decision, offset);
    decision["warp_decision"]["planner_replay_capsule"]["context"]["candidates"][post]["watch"]
        ["Learned"]["play_start_p95_ms"]
        .as_u64()
        .expect("learned deadline")
}

fn post_at(decision: &serde_json::Value, offset: i32) -> &str {
    decision["replay_state"]["candidates"]
        .as_array()
        .expect("candidates")
        .iter()
        .find(|candidate| candidate["feed_offset"] == offset)
        .expect("next video")["post"]
        .as_str()
        .expect("next identity")
}
