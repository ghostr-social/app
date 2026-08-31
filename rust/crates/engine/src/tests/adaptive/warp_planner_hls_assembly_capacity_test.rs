use crate::adaptive::{
    AdaptivePlayabilityPolicy, FeedOffset, HlsBootstrapStage, HlsBootstrapState,
    HlsCandidateSnapshot, HlsObjectCursor, HlsTransport, PlannerContext, ViewProbability,
    WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

const REQUIRED: u64 = 344 * 1024;

#[test]
fn final_hls_tail_requires_tail_plus_transactional_assembly_storage() {
    let mut state = snapshot(0, 80_000_000, 0, 0);
    state.hls_candidates.push(candidate());
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let below = context(&state, REQUIRED - 1);
    let exact = context(&state, REQUIRED);

    let blocked = plan(&state, &base, &below);
    let admitted = plan(&state, &base, &exact);

    assert_eq!(
        blocked.generated.actions[0].node.resources.storage_bytes,
        44 * 1024
    );
    assert!(blocked.admissible_action_ids.is_empty());
    assert_eq!(admitted.admissible_action_ids.len(), 1);
}

fn context(state: &crate::adaptive::PlayabilitySnapshot, bytes: u64) -> PlannerContext {
    PlannerContext::explicitly_unavailable(state).with_segmented_storage_available_bytes(bytes)
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

fn candidate() -> HlsCandidateSnapshot {
    HlsCandidateSnapshot {
        post: PostId::new("tail"),
        feed_offset: FeedOffset::new(0),
        view_probability: ViewProbability::new(1.0).expect("valid test fixture"),
        startup_value_ms: 750,
        cursor: HlsObjectCursor::new(7, 256 * 1024, Some(300 * 1024), HlsTransport::ResumeRange),
        player_preparation: Default::default(),
        state: HlsBootstrapState::Pending {
            stage: HlsBootstrapStage::Initialization,
            source: "https://tail.example/init.mp4".into(),
        },
    }
}
