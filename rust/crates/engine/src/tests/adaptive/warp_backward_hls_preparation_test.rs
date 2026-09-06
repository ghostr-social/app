use super::warp_backward_fixture::state;
use crate::adaptive::{
    AdaptivePlayabilityPolicy, FeedOffset, HlsBootstrapStage, HlsBootstrapState,
    HlsCandidateSnapshot, HlsObjectCursor, HlsTransport, PlannerCommand, PlannerContext,
    ViewProbability, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::PostId;

#[test]
fn a_previous_hls_initialization_has_a_bounded_admissible_preparation() {
    let mut state = state();
    state.candidates.truncate(1);
    state.hls_candidates.push(previous());
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let context = PlannerContext::explicitly_unavailable(&state)
        .with_segmented_storage_available_bytes(256 * 1024);
    let result = WarpPlanner::default().plan(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &context,
    ));
    let action = result
        .generated
        .actions
        .iter()
        .find(|action| matches!(action.command, PlannerCommand::FetchHlsBootstrap { .. }))
        .expect("a reverse swipe needs the retained HLS initialization dependency");
    assert!(result.admissible_action_ids.contains(&action.node.id));
    assert_eq!(action.node.authorized_resources().network_bytes, 32 * 1024);
    assert!(
        result.semantic.is_empty(),
        "history does not enter the forward rescue pool"
    );
}

fn previous() -> HlsCandidateSnapshot {
    HlsCandidateSnapshot {
        post: PostId::new("previous"),
        feed_offset: FeedOffset::new(-1),
        view_probability: ViewProbability::new(0.9).expect("probability"),
        startup_value_ms: 2_000,
        cursor: HlsObjectCursor::new(1, 0, Some(32 * 1024), HlsTransport::Start),
        player_preparation: Default::default(),
        state: HlsBootstrapState::Pending {
            stage: HlsBootstrapStage::Initialization,
            source: "https://hls.example/init.mp4".into(),
        },
    }
}
