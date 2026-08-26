use super::{HedgeCase, ALTERNATE, OBSERVED_AT_MS, PRIMARY};
use ghostr_engine::host_stats::{HostStats, ThroughputSample};
use ghostr_engine::origin_model::{
    ErrorReason, MediaClass, NetworkClass, OriginContext, OriginObservation, OriginQuery,
    RequestMethod,
};
use core::time::Duration;

pub(super) fn model(case: HedgeCase) -> HostStats {
    let mut stats = HostStats::new();
    let sample = ThroughputSample::new(2_000_000, Duration::from_secs(1), 1, 1).expect("valid test fixture");
    stats.record_overall_throughput(sample);
    observe(&mut stats, PRIMARY, 900, 500_000);
    observe(&mut stats, ALTERNATE, 20, 20_000_000);
    observe_class(&mut stats, PRIMARY, NetworkClass::Wifi, 1_200, 1_000_000);
    observe_class(&mut stats, PRIMARY, NetworkClass::Cellular, 2_000, 100_000);
    if matches!(case, HedgeCase::PrimaryUnavailable) {
        block_primary(&mut stats);
    }
    stats
}

fn block_primary(stats: &mut HostStats) {
    let query = query(PRIMARY, NetworkClass::Unavailable);
    for _ in 0..3 {
        let observation =
            OriginObservation::failure(query.clone(), OBSERVED_AT_MS, ErrorReason::Timeout);
        stats.origin_model_mut().observe(&observation);
    }
}

fn observe(stats: &mut HostStats, source: &str, ttfb_ms: u64, throughput_bps: u64) {
    observe_query(
        stats,
        &query(source, NetworkClass::Unavailable),
        ttfb_ms,
        throughput_bps,
    );
}

fn observe_class(
    stats: &mut HostStats,
    source: &str,
    network: NetworkClass,
    ttfb_ms: u64,
    throughput_bps: u64,
) {
    observe_query(stats, &query(source, network), ttfb_ms, throughput_bps);
}

fn observe_query(stats: &mut HostStats, query: &OriginQuery, ttfb_ms: u64, throughput_bps: u64) {
    for _ in 0..64 {
        let observation = OriginObservation::success(query.clone(), OBSERVED_AT_MS)
            .with_range_compliance(true)
            .with_ttfb_ms(ttfb_ms)
            .with_throughput_bps(throughput_bps);
        stats.origin_model_mut().observe(&observation);
    }
}

fn query(source: &str, network: NetworkClass) -> OriginQuery {
    let context = OriginContext::new(RequestMethod::RangeGet, 64_000, MediaClass::ProgressiveMp4)
        .with_network(network)
        .with_concurrency(1)
        .with_observed_at_ms(OBSERVED_AT_MS);
    OriginQuery::new(source, context)
}
