use super::super::PlanInputs;
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{NetworkSnapshot, OriginHealth};
use ghostr_engine::evidence::{EvidenceField, EvidenceValue};
use ghostr_engine::host_stats::OPTIMISTIC_THROUGHPUT_BPS;
use ghostr_engine::origin_model::{
    Admission, DecisionMode, MediaClass, NetworkClass, OriginContext, OriginQuery, RequestMethod,
};
use ghostr_engine::playback::EstimateConfidence;
use ghostr_engine::{PostId, RequestAuthority};

const BOOTSTRAP_REQUEST_BYTES: u64 = 256 * 1024;

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
        .filter_map(|url| origin(inputs, entry, url, state.network_class()))
        .collect()
}

fn origin(
    inputs: &PlanInputs<'_>,
    entry: &ghostr_engine::catalog::CatalogEntry,
    url: String,
    network_class: NetworkClass,
) -> Option<OriginHealth> {
    let query = origin_query(inputs, entry, &url, network_class)?;
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
    network_class: NetworkClass,
) -> Option<OriginQuery> {
    let authority = RequestAuthority::from_url(url)?;
    let (method, media, bytes) = request_context(entry, url, inputs.observed_at_ms);
    let concurrency = active_on_authority(inputs, &authority).saturating_add(1);
    Some(OriginQuery::new(
        url,
        OriginContext::new(method, bytes, media)
            .with_network(network_class)
            .with_concurrency(concurrency)
            .with_observed_at_ms(inputs.observed_at_ms),
    ))
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
    observed_at_ms: u64,
) -> (RequestMethod, MediaClass, u64) {
    if entry.meta.delivery == ghostr_engine::DeliveryKind::Hls {
        return (
            RequestMethod::SegmentGet,
            MediaClass::Segmented,
            BOOTSTRAP_REQUEST_BYTES,
        );
    }
    let assessment = entry.evidence_assessment_for(url, observed_at_ms);
    match assessment.value(EvidenceField::RangeSupport) {
        Some(EvidenceValue::RangeSupport(false)) => (
            RequestMethod::FullGet,
            MediaClass::WholeObject,
            whole_request_bytes(entry, url, &assessment),
        ),
        _ => (
            RequestMethod::RangeGet,
            MediaClass::ProgressiveMp4,
            BOOTSTRAP_REQUEST_BYTES,
        ),
    }
}

fn whole_request_bytes(
    entry: &ghostr_engine::catalog::CatalogEntry,
    url: &str,
    assessment: &ghostr_engine::evidence::EvidenceAssessment,
) -> u64 {
    assessment.size.exact.unwrap_or_else(|| {
        [
            assessment.size.upper,
            entry.meta.size_bytes,
            entry.planning_total_for(url),
        ]
        .into_iter()
        .flatten()
        .max()
        .unwrap_or(BOOTSTRAP_REQUEST_BYTES)
    })
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

#[cfg(test)]
#[path = "telemetry/request_context_conflict_test.rs"]
mod request_context_conflict_test;
#[cfg(test)]
#[path = "telemetry/request_context_evidence_test.rs"]
mod request_context_evidence_test;
