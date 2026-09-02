use crate::adaptive::{
    AdaptivePlayabilityPolicy, AllocationPlan, FeedOffset, PlannerContext, ViewProbability,
};
use crate::origin_model::{
    MediaClass, NetworkClass, OriginContext, OriginModel, OriginObservation, OriginQuery,
    RequestMethod,
};
use crate::tests::adaptive_support::snapshot;

const SOURCE: &str = "https://same.example/video.mp4";

pub(super) struct NetworkClassFixture {
    pub(super) snapshot: crate::adaptive::PlayabilitySnapshot,
    pub(super) base: AllocationPlan,
    pub(super) origins: OriginModel,
    pub(super) context: PlannerContext,
}

#[derive(Clone, Copy)]
pub(super) struct NetworkEvidence {
    class: NetworkClass,
    rate: u64,
    ttfb: u64,
    observed_at_ms: u64,
}

pub(super) fn fixture(network_class: NetworkClass) -> NetworkClassFixture {
    let mut snapshot = snapshot(1, 80_000_000, 20_000, 0);
    for candidate in &mut snapshot.candidates {
        candidate.feed_offset = FeedOffset::new(0);
        candidate.view_probability = ViewProbability::new(1.0).expect("valid test fixture");
        candidate.origins[0].source = SOURCE.to_owned();
    }
    let base = AdaptivePlayabilityPolicy.plan(&snapshot);
    let context =
        PlannerContext::explicitly_unavailable(&snapshot).with_network_class(network_class);
    NetworkClassFixture {
        snapshot,
        base,
        origins: evidenced_origins(),
        context,
    }
}

fn evidenced_origins() -> OriginModel {
    let mut model = OriginModel::default();
    for sample in 0..24 {
        record(
            &mut model,
            SOURCE,
            evidence(NetworkClass::Wifi, 80_000_000, 20, sample),
        );
        record(
            &mut model,
            SOURCE,
            evidence(NetworkClass::Cellular, 1_000_000, 400, sample),
        );
    }
    model
}

const fn evidence(class: NetworkClass, rate: u64, ttfb: u64, sample: u64) -> NetworkEvidence {
    NetworkEvidence {
        class,
        rate,
        ttfb,
        observed_at_ms: 9_000 + sample,
    }
}

fn record(model: &mut OriginModel, source: &str, evidence: NetworkEvidence) {
    for (method, bytes) in [
        (
            RequestMethod::RangeGet,
            crate::adaptive::REQUEST_SLICE_BYTES,
        ),
        (RequestMethod::FullGet, 3_750_000),
    ] {
        let context = OriginContext::new(method, bytes, MediaClass::ProgressiveMp4)
            .with_network(evidence.class)
            .with_observed_at_ms(evidence.observed_at_ms);
        model.observe(
            &OriginObservation::success(OriginQuery::new(source, context), evidence.observed_at_ms)
                .with_ttfb_ms(evidence.ttfb)
                .with_throughput_bps(evidence.rate),
        );
    }
}
