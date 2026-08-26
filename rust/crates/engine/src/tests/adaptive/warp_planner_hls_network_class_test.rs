use crate::adaptive::{
    AdaptivePlayabilityPolicy, FeedOffset, HlsBootstrapStage, HlsBootstrapState,
    HlsCandidateSnapshot, PlannerContext, ViewProbability, WarpPlanner, WarpPlannerInput,
};
use crate::origin_model::{
    MediaClass, NetworkClass, OriginContext, OriginModel, OriginObservation, OriginQuery,
    RequestMethod,
};
use crate::tests::adaptive_support::snapshot;
use crate::PostId;

const SOURCE: &str = "https://same.example/init.mp4";

#[test]
fn hls_prediction_uses_the_single_plan_network_class_evidence_bucket() {
    let mut state = snapshot(0, 80_000_000, 0, 0);
    state.hls_candidates = vec![candidate("post")];
    let origins = network_evidence();
    let base = AdaptivePlayabilityPolicy.plan(&state);
    let completion = |network_class| {
        let context = PlannerContext::explicitly_unavailable(&state)
            .with_segmented_storage_available_bytes(8 * 1024 * 1024)
            .with_network_class(network_class);
        WarpPlanner::default()
            .plan(WarpPlannerInput::new(&state, &base, &origins, &context))
            .generated
            .actions[0]
            .node
            .forecast
            .completion
            .expected_ms
    };

    assert!(completion(NetworkClass::Wifi) < completion(NetworkClass::Cellular));
}

fn candidate(post: &str) -> HlsCandidateSnapshot {
    HlsCandidateSnapshot {
        post: PostId::new(post),
        feed_offset: FeedOffset::new(0),
        view_probability: ViewProbability::new(1.0).expect("valid test fixture"),
        startup_value_ms: 750,
        cursor: Default::default(),
        state: HlsBootstrapState::Pending {
            stage: HlsBootstrapStage::Initialization,
            source: SOURCE.to_owned(),
        },
    }
}

fn network_evidence() -> OriginModel {
    let mut model = OriginModel::default();
    for sample in 0..24 {
        observe(&mut model, NetworkClass::Wifi, 80_000_000, 20, sample);
        observe(&mut model, NetworkClass::Cellular, 1_000_000, 400, sample);
    }
    model
}

fn observe(model: &mut OriginModel, class: NetworkClass, rate: u64, ttfb: u64, sample: u64) {
    let context = OriginContext::new(
        RequestMethod::SegmentGet,
        crate::adaptive::REQUEST_SLICE_BYTES,
        MediaClass::Segmented,
    )
    .with_concurrency(1)
    .with_network(class)
    .with_observed_at_ms(9_000 + sample);
    model.observe(
        &OriginObservation::success(OriginQuery::new(SOURCE, context), 9_000 + sample)
            .with_ttfb_ms(ttfb)
            .with_throughput_bps(rate),
    );
}
