use super::support::{planned, record};
use crate::adaptive::{
    ActionKind, AdaptivePlayabilityPolicy, BeamConfig, DecisionReplayStatus, PlannerContext,
    PlannerRetryAvailability, PlannerWatchEvidence, WarpPlanner, WarpPlannerConfig,
    WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;

#[test]
fn complete_capsule_reruns_real_multi_action_transform_rescue_plan() {
    let (state, decision) = planned();
    assert!(decision.generated.actions.len() > 1);
    assert!(decision
        .generated
        .actions
        .iter()
        .any(|action| matches!(action.node.kind, ActionKind::Transform(_))));

    let captured = record(&state, &decision);
    let replay = captured.replay_warp_search().expect("full planner replay");

    assert_eq!(captured.replay(), DecisionReplayStatus::Verified);
    assert_eq!(replay.decision(), captured.warp_decision.as_ref().unwrap());
    assert!(serde_json::to_string(&captured)
        .unwrap()
        .contains("planner_replay_capsule"));
}

#[test]
fn privacy_hash_order_reversal_preserves_original_candidate_order() {
    let state = snapshot(2, 20_000_000, 8_000, 18);
    assert_eq!(state.candidates[0].post.as_str(), "p0");
    assert_eq!(state.candidates[1].post.as_str(), "p1");
    let context = PlannerContext::explicitly_unavailable(&state)
        .with_retry_availability(
            state.candidates[0].post.clone(),
            PlannerRetryAvailability::Cooling { eligible_at_ms: 10 },
        )
        .with_retry_availability(
            state.candidates[1].post.clone(),
            PlannerRetryAvailability::Cooling { eligible_at_ms: 20 },
        )
        .with_watch(
            state.candidates[1].post.clone(),
            PlannerWatchEvidence::learned(4_000, 4_000, 8_000, 12_000, 2_000, None),
        );
    let config = WarpPlannerConfig {
        beam: BeamConfig::new(2, 8, 64, u64::MAX),
        ..WarpPlannerConfig::default()
    };
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let decision = WarpPlanner::new(config).plan(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &context,
    ));
    let captured = super::support::record(&state, &decision);
    let retry = &captured.warp_decision.as_ref().unwrap().retry_availability;

    assert!(
        retry[0].post_id > retry[1].post_id,
        "fixture reverses hash order"
    );
    let json = serde_json::to_string(&captured).unwrap();
    assert!(json.contains("play_start_p95_ms"));
    assert!(!json.contains("\"p1\""));
    assert!(captured.replay_warp_search().is_ok());
}
