use ghostr_delivery::delivery_events::{DeliveryHandle, PlanEvidence};

pub fn assert_serialized_correlation(handle: &DeliveryHandle, plan: &PlanEvidence) {
    let sequence = plan.decision_sequence.expect("linked decision sequence");
    let page = json(handle.evidence_page_json(0, usize::MAX));
    let plans = page["plan_page"]["records"]
        .as_array()
        .expect("serialized plans");
    let serialized = plans
        .iter()
        .find(|value| value["revision"].as_u64() == Some(plan.revision))
        .expect("serialized linked plan");
    assert_eq!(serialized["decision_sequence"], sequence);
    assert_eq!(serialized["observed_at_ms"], plan.observed_at_ms);

    let history = json(handle.decision_history_json());
    let decisions = history["decisions"]["records"]
        .as_array()
        .expect("serialized decisions");
    let decision = decisions
        .iter()
        .find(|value| value["sequence"].as_u64() == Some(sequence))
        .expect("serialized linked decision");
    assert_eq!(
        decision["replay_state"]["observed_at_ms"],
        plan.observed_at_ms
    );
}

fn json(encoded: serde_json::Result<String>) -> serde_json::Value {
    let encoded = encoded.expect("evidence JSON");
    serde_json::from_str(&encoded).expect("valid evidence JSON")
}
