use super::support::{capsule, planned, record};
use crate::adaptive::{
    AdaptivePlayabilityPolicy, DecisionRecord, DecisionReplayStatus, PlannerContext,
    ResourceFeedback, ResourceObservation, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn absent_capsule_keeps_legacy_v2_bytes_and_captured_search_replay() {
    let (state, mut decision) = planned();
    decision.planner_replay = None;
    let captured = record(&state, &decision);
    let json = serde_json::to_string(&captured).unwrap();
    let restored: DecisionRecord = serde_json::from_str(&json).unwrap();

    assert!(!json.contains("planner_replay_capsule"));
    assert_eq!(serde_json::to_string(&restored).unwrap(), json);
    assert!(restored.replay_warp_search().is_ok());
}

#[test]
fn authentically_incomplete_capsule_fails_closed() {
    let (state, mut decision) = planned();
    capsule(&mut decision).mark_incomplete();
    let captured = record(&state, &decision);

    assert_eq!(captured.replay(), DecisionReplayStatus::Verified);
    assert_eq!(
        captured.replay_warp_search(),
        Err(DecisionReplayStatus::AdvancedReplayUnavailable)
    );
}

#[test]
fn oversized_real_planner_input_records_unavailable_capsule() {
    let mut state = snapshot(65, 20_000_000, 8_000, 18);
    for (index, candidate) in state.candidates.iter_mut().enumerate() {
        candidate.origins[0].source = format!("https://origin-{index}.example/media");
    }
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let context = PlannerContext::explicitly_unavailable(&state);
    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &context,
    ));
    let captured = record(&state, &decision);

    assert_eq!(
        captured.replay(),
        DecisionReplayStatus::AdvancedReplayUnavailable
    );
    assert_eq!(
        captured.replay_warp_search(),
        Err(DecisionReplayStatus::AdvancedReplayUnavailable)
    );
}

#[test]
fn pre_revision_feedback_capsule_keeps_its_historical_json_shape() {
    let state = snapshot(2, 20_000_000, 8_000, 18);
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let context = PlannerContext::explicitly_unavailable(&state).with_feedback(ResourceFeedback {
        revision: 0,
        actual: ResourceObservation::new(200, 0, 0, 1),
        target: ResourceObservation::new(100, 0, 0, 1),
        price_snapshot: None,
    });
    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &context,
    ));
    let captured = record(&state, &decision);
    let json = serde_json::to_string(&captured).unwrap();
    let restored: DecisionRecord = serde_json::from_str(&json).unwrap();

    assert!(!json.contains("\"revision\""));
    assert_eq!(serde_json::to_string(&restored).unwrap(), json);
    assert_eq!(restored.replay(), DecisionReplayStatus::Verified);
    assert!(restored.replay_warp_search().is_ok());
}
