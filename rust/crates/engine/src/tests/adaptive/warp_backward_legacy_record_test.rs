use crate::adaptive::RecordedPlannerReplayCapsule;

#[test]
fn records_without_a_navigation_policy_keep_their_legacy_shape() {
    let legacy = serde_json::json!({
        "complete": false,
        "config": {
            "beam_depth": 4, "beam_width": 32, "beam_expansions": 256,
            "beam_latency_us": 2000, "twin_particles": 48, "twin_tail_bps": 9500,
            "semantic_top_k": 5, "semantic_epsilon_micros": 0,
            "safety_rescue_bps": 9500, "emergency_rescue_bps": 9900
        }
    });
    let restored: RecordedPlannerReplayCapsule =
        serde_json::from_value(legacy.clone()).expect("legacy planner capsule decodes");
    let encoded = serde_json::to_value(restored).expect("legacy planner capsule encodes");
    assert!(encoded["config"]
        .get("navigation_preparation_policy")
        .is_none());
    assert_eq!(encoded, legacy);
}
