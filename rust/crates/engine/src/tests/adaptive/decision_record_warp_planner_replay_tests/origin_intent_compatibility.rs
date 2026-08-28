use super::super::reserve_support;
use crate::adaptive::{DecisionRecord, DecisionReplayStatus, WarpPlanner, WarpPlannerInput};

#[test]
fn marker_free_origin_admission_record_replays_legacy_delivery_actions() {
    let state = reserve_support::rescue_state();
    let base = reserve_support::safety_plan();
    let context = reserve_support::rescue_context(&state);
    let origins = reserve_support::reliable_origin();
    let decision = WarpPlanner::new(reserve_support::replay_config())
        .plan_legacy_origin_admission_for_test(WarpPlannerInput::new(
            &state, &base, &origins, &context,
        ));
    let captured = reserve_support::record(&state, &decision);
    let encoded = serde_json::to_string(&captured).expect("valid legacy fixture");

    assert!(!encoded.contains("origin_admission_intent"));
    assert!(!encoded.contains("origin_admission_generation_policy"));
    let restored: DecisionRecord = serde_json::from_str(&encoded).expect("legacy record");
    assert_eq!(
        serde_json::to_string(&restored).expect("stable legacy fixture"),
        encoded
    );
    assert_eq!(restored.integrity_status(), DecisionReplayStatus::Verified);
    assert!(restored.replay_warp_search().is_ok());
}
