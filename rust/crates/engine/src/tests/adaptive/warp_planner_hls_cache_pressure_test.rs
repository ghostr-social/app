use crate::adaptive::{
    AdaptivePlayabilityPolicy, FeedOffset, HlsBootstrapStage, HlsBootstrapState,
    HlsCandidateSnapshot, PlannerContext, ViewProbability, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

#[test]
fn global_segmented_headroom_prunes_each_infeasible_hls_stage() {
    let mut state = snapshot(0, 80_000_000, 0, 0);
    state.hls_candidates = ["first", "second"].into_iter().map(candidate).collect();
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let context = PlannerContext::explicitly_unavailable(&state)
        .with_segmented_storage_available_bytes(8 * 1024 * 1024 - 1);

    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &context,
    ));

    assert_eq!(decision.generated.actions.len(), 2);
    assert!(decision.admissible_action_ids.is_empty());
    assert!(decision.selected.is_none());
}

#[test]
fn omitted_segmented_headroom_fails_closed_until_exact_capacity_is_explicit() {
    let mut state = snapshot(0, 80_000_000, 0, 0);
    state.hls_candidates.push(candidate("only"));
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let default = PlannerContext::explicitly_unavailable(&state);
    let json = serde_json::to_value(default).unwrap();
    assert!(json.get("segmented_storage_available_bytes").is_none());
    let omitted: PlannerContext = serde_json::from_value(json).unwrap();
    let blocked = plan(&state, &base, &omitted);
    let exact = PlannerContext::explicitly_unavailable(&state)
        .with_segmented_storage_available_bytes(8 * 1024 * 1024);
    let admitted = plan(&state, &base, &exact);

    assert_eq!(blocked.generated.actions.len(), 1);
    assert!(blocked.admissible_action_ids.is_empty());
    assert_eq!(admitted.admissible_action_ids.len(), 1);
}

fn plan(
    state: &crate::adaptive::PlayabilitySnapshot,
    base: &crate::adaptive::AllocationPlan,
    context: &PlannerContext,
) -> crate::adaptive::WarpPlanningDecision {
    WarpPlanner::default().plan(WarpPlannerInput::new(
        state,
        base,
        &OriginModel::default(),
        context,
    ))
}

fn candidate(post: &str) -> HlsCandidateSnapshot {
    HlsCandidateSnapshot {
        post: PostId::new(post),
        feed_offset: FeedOffset::new(0),
        view_probability: ViewProbability::new(1.0).unwrap(),
        startup_value_ms: 750,
        state: HlsBootstrapState::Pending {
            stage: HlsBootstrapStage::Initialization,
            source: format!("https://{post}.example/init.mp4"),
        },
    }
}
