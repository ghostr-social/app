use crate::adaptive::{
    AdaptivePlayabilityPolicy, FeedOffset, HlsBootstrapStage, HlsBootstrapState,
    HlsCandidateSnapshot, PlannerContext, ViewProbability, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::OriginModel;
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

#[test]
fn current_initialization_stage_remains_worth_selecting_at_the_production_envelope() {
    let mut state = snapshot(0, 33_554_432, 0, 0);
    state.network.rtt_ms = 250;
    state.hls_candidates.push(HlsCandidateSnapshot {
        post: PostId::new("p0"),
        feed_offset: FeedOffset::new(0),
        view_probability: ViewProbability::new(1.0).expect("valid test fixture"),
        startup_value_ms: 750,
        cursor: Default::default(),
        player_preparation: Default::default(),
        state: HlsBootstrapState::Pending {
            stage: HlsBootstrapStage::Initialization,
            source: "https://hls.example/init.mp4".to_owned(),
        },
    });
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let mut context = PlannerContext::explicitly_unavailable(&state)
        .with_segmented_storage_available_bytes(8 * 1024 * 1024);
    let mut limits = context.limits;
    limits.network_burst_bytes = 18 * 1024 * 1024;
    context = context.with_limits(limits);

    let decision = WarpPlanner::default().plan(WarpPlannerInput::new(
        &state,
        &base,
        &OriginModel::default(),
        &context,
    ));

    assert_eq!(
        decision.selected.map(|selected| selected.command),
        Some(crate::adaptive::PlannerCommand::FetchHlsBootstrap {
            post: PostId::new("p0"),
            stage: HlsBootstrapStage::Initialization,
            source: "https://hls.example/init.mp4".to_owned(),
            cursor: Default::default(),
            maximum_bytes: crate::adaptive::REQUEST_SLICE_BYTES,
            committed_until_ms: 13_000,
        })
    );
}
