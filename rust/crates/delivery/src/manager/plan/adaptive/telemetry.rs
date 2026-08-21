use super::super::PlanInputs;
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{NetworkSnapshot, OriginHealth};
use ghostr_engine::host_stats::OPTIMISTIC_THROUGHPUT_BPS;
use ghostr_engine::origin_model::{
    Admission, DecisionMode, MediaClass, NetworkClass, OriginContext, OriginQuery, RequestMethod,
};
use ghostr_engine::playback::EstimateConfidence;
use ghostr_engine::{PostId, RequestAuthority};

pub(super) fn origins(
    state: &DeliveryState,
    inputs: &PlanInputs<'_>,
    post: &PostId,
) -> Vec<OriginHealth> {
    let Some(entry) = state.catalog().lookup(post) else {
        return Vec::new();
    };
    inputs
        .retry
        .live_urls(post, &entry.meta.urls)
        .into_iter()
        .filter_map(|url| origin(inputs, entry, url))
        .collect()
}

fn origin(
    inputs: &PlanInputs<'_>,
    entry: &ghostr_engine::catalog::CatalogEntry,
    url: String,
) -> Option<OriginHealth> {
    let authority = RequestAuthority::from_url(&url)?;
    let query = origin_query(inputs, entry, &url, &authority);
    let estimate =
        inputs
            .stats
            .origin_model()
            .estimate(&query, inputs.observed_at_ms, DecisionMode::Safety);
    let reliability = estimate.success.selected
        * estimate
            .range_compliance
            .map_or(1.0, |range| range.selected);
    let failure = ((1.0 - reliability) * 10_000.0).round() as u16;
    let admission = inputs
        .stats
        .origin_model()
        .circuit_admission(&query, inputs.observed_at_ms);
    Some(OriginHealth {
        source: url,
        available: admission != Admission::Blocked,
        throughput_bps: finite_bits(estimate.throughput_bps.selected as f64),
        rtt_ms: estimate.ttfb_ms.selected,
        packet_loss_bps: inputs.packet_loss_bps,
        failure_bps: failure,
    })
}

fn origin_query(
    inputs: &PlanInputs<'_>,
    entry: &ghostr_engine::catalog::CatalogEntry,
    url: &str,
    authority: &RequestAuthority,
) -> OriginQuery {
    let (method, media, bytes) = request_context(entry, url);
    let concurrency = active_on_authority(inputs, authority).saturating_add(1);
    OriginQuery::new(
        url,
        OriginContext::new(method, bytes, media)
            .with_network(NetworkClass::Unavailable)
            .with_concurrency(concurrency)
            .with_observed_at_ms(inputs.observed_at_ms),
    )
}

fn active_on_authority(inputs: &PlanInputs<'_>, authority: &RequestAuthority) -> usize {
    let bodies = inputs
        .in_flight
        .iter()
        .map(|active| active.identity().source().as_str());
    let probes = inputs
        .active_head_probes
        .iter()
        .map(|identity| identity.source().as_str());
    bodies
        .chain(probes)
        .filter(|source| RequestAuthority::from_url(source).as_ref() == Some(authority))
        .count()
}

fn request_context(
    entry: &ghostr_engine::catalog::CatalogEntry,
    url: &str,
) -> (RequestMethod, MediaClass, u64) {
    if entry.meta.delivery == ghostr_engine::DeliveryKind::Hls {
        return (RequestMethod::SegmentGet, MediaClass::Segmented, 256 * 1024);
    }
    match entry.observed_range_support_for(url) {
        Some(false) => (
            RequestMethod::FullGet,
            MediaClass::WholeObject,
            entry.planning_total_for(url).unwrap_or(256 * 1024),
        ),
        _ => (
            RequestMethod::RangeGet,
            MediaClass::ProgressiveMp4,
            256 * 1024,
        ),
    }
}

pub(super) fn network(inputs: &PlanInputs<'_>) -> NetworkSnapshot {
    let estimate = inputs.stats.overall_throughput();
    let throughput = estimate.map_or(OPTIMISTIC_THROUGHPUT_BPS, |value| value.bytes_per_second());
    NetworkSnapshot {
        throughput_bps: finite_bits(throughput),
        rtt_ms: inputs
            .stats
            .overall_ttfb()
            .map_or(250, |value| value.as_millis() as u64),
        packet_loss_bps: inputs.packet_loss_bps,
        connection_capacity: inputs.connection_capacity.max(1),
        connection_ceiling: inputs.connection_ceiling.max(1),
        per_authority_request_limit: inputs.per_authority_request_limit.max(1),
        confidence: estimate.map_or(EstimateConfidence::Low, |value| {
            EstimateConfidence::from_evidence(
                value.sample_count(),
                value.last_observed_at_ms(),
                inputs.observed_at_ms,
            )
        }),
    }
}

fn finite_bits(bytes_per_second: f64) -> u64 {
    if !bytes_per_second.is_finite() || bytes_per_second <= 0.0 {
        return 0;
    }
    (bytes_per_second * 8.0).min(u64::MAX as f64).round() as u64
}
