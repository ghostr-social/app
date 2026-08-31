use crate::adaptive::axiom_test_support::WarpActionGenerator;
use crate::adaptive::{
    AdaptivePlayabilityPolicy, FeedOffset, HlsBootstrapStage, HlsBootstrapState,
    HlsCandidateSnapshot, PlannerCommand, PlannerContext, ViewProbability, WarpPlanner,
    WarpPlannerInput,
};
use crate::origin_model::{
    MediaClass, NetworkClass, OriginContext, OriginModel, OriginObservation, OriginQuery,
    RequestMethod,
};
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

const FAST: &str = "https://fast.example/init.mp4";
const SLOW: &str = "https://slow.example/init.mp4";

#[test]
fn faster_hls_stage_wins_over_equal_slow_stage_with_strong_evidence() {
    let mut state = snapshot(0, 80_000_000, 0, 0);
    state.hls_candidates = vec![candidate("slow", SLOW), candidate("fast", FAST)];
    let origins = evidenced_origins();
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let context = PlannerContext::explicitly_unavailable(&state)
        .with_segmented_storage_available_bytes(16 * 1024 * 1024)
        .with_network_class(NetworkClass::Wifi);
    let generated = WarpActionGenerator::generate(&state, &base, &origins, &context);

    assert_eq!(
        generated.actions[0].node.value.reserve_gain_micros,
        generated.actions[1].node.value.reserve_gain_micros,
        "equal user progress cannot reward a slower prediction"
    );

    let decision =
        WarpPlanner::default().plan(WarpPlannerInput::new(&state, &base, &origins, &context));

    assert!(matches!(
        decision.selected.map(|selected| selected.command),
        Some(PlannerCommand::FetchHlsBootstrap { source, .. }) if source == FAST
    ));
}

fn candidate(post: &str, source: &str) -> HlsCandidateSnapshot {
    HlsCandidateSnapshot {
        post: PostId::new(post),
        feed_offset: FeedOffset::new(0),
        view_probability: ViewProbability::new(1.0).expect("valid test fixture"),
        startup_value_ms: 750,
        cursor: Default::default(),
        player_preparation: Default::default(),
        state: HlsBootstrapState::Pending {
            stage: HlsBootstrapStage::Initialization,
            source: source.to_owned(),
        },
    }
}

fn evidenced_origins() -> OriginModel {
    let mut model = OriginModel::default();
    for sample in 0..24 {
        observe(&mut model, FAST, 80_000_000, 20, sample);
        observe(&mut model, SLOW, 1_000_000, 400, sample);
    }
    model
}

fn observe(model: &mut OriginModel, source: &str, rate: u64, ttfb: u64, sample: u64) {
    let context = OriginContext::new(
        RequestMethod::SegmentGet,
        crate::adaptive::REQUEST_SLICE_BYTES,
        MediaClass::Segmented,
    )
    .with_concurrency(1)
    .with_network(NetworkClass::Wifi)
    .with_observed_at_ms(9_000 + sample);
    let observation = OriginObservation::success(OriginQuery::new(source, context), 9_000 + sample)
        .with_ttfb_ms(ttfb)
        .with_throughput_bps(rate);
    model.observe(&observation);
}
