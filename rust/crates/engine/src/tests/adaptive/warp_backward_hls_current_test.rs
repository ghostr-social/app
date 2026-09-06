use super::warp_backward_fixture::state;
use crate::adaptive::{
    AdaptivePlayabilityPolicy, FeedOffset, HlsBootstrapStage, HlsBootstrapState,
    HlsCandidateSnapshot, HlsObjectCursor, HlsTransport, PlannerContext, PlannerLimits,
    ViewProbability, WarpPlanner, WarpPlannerInput, REQUEST_SLICE_BYTES,
};
use crate::origin_model::OriginModel;

#[test]
fn history_cannot_take_the_only_request_needed_by_current_hls_initialization() {
    let mut state = state();
    state.candidates.remove(0);
    state.hls_candidates.push(current(&state.playback.current));
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let context = PlannerContext::explicitly_unavailable(&state)
        .with_segmented_storage_available_bytes(REQUEST_SLICE_BYTES)
        .with_limits(PlannerLimits {
            network_burst_bytes: REQUEST_SLICE_BYTES,
            network_rate_bytes_per_second: REQUEST_SLICE_BYTES,
            cpu_ms: 0,
            request_tokens: 1,
            per_origin_requests: 1,
        });
    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &context,
    ));
    let history = &state.candidates[0].post;
    assert!(decision
        .generated
        .actions
        .iter()
        .any(|action| &action.node.post == history));
    assert!(decision
        .generated
        .actions
        .iter()
        .filter(|action| &action.node.post == history)
        .all(|action| !decision.admissible_action_ids.contains(&action.node.id)));
}

fn current(post: &crate::PostId) -> HlsCandidateSnapshot {
    HlsCandidateSnapshot {
        post: post.clone(),
        feed_offset: FeedOffset::new(0),
        view_probability: ViewProbability::new(1.0).expect("probability"),
        startup_value_ms: 2_000,
        cursor: HlsObjectCursor::new(1, 0, Some(32 * 1024), HlsTransport::Start),
        player_preparation: Default::default(),
        state: HlsBootstrapState::Pending {
            stage: HlsBootstrapStage::Initialization,
            source: "https://current.example/init.mp4".into(),
        },
    }
}
