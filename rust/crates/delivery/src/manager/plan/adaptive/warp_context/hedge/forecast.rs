use super::super::ActiveContextInput;
use ghostr_engine::adaptive::{CandidateSnapshot, OriginHealth, RetrievalRequest};
use ghostr_engine::origin_model::{
    DecisionMode, MediaClass, NetworkClass, OriginContext, OriginEstimate, OriginQuery,
    RequestMethod,
};
use ghostr_engine::RequestAuthority;

const MICROS_PER_SECOND: u128 = 1_000_000;

pub(super) struct TailComparison {
    pub(super) trigger_ms: u64,
    pub(super) loss_reduction_micros: u64,
    pub(super) duplicate_cost_micros: u64,
}

pub(super) fn compare(
    evidence: &ActiveContextInput<'_>,
    candidate: &CandidateSnapshot,
    primary: &OriginHealth,
    alternate: &OriginHealth,
) -> TailComparison {
    let primary = estimate(evidence, candidate, &primary.source, true);
    let alternate = estimate(evidence, candidate, &alternate.source, false);
    let bytes = evidence.active.request().reserved_network_bytes();
    let primary_p95 = p95_micros(&primary, bytes);
    let alternate_p95 = p95_micros(&alternate, bytes);
    TailComparison {
        trigger_ms: micros_to_ms(primary_p95),
        loss_reduction_micros: primary_p95.saturating_sub(alternate_p95),
        duplicate_cost_micros: transfer_micros(bytes, alternate.throughput_bps.p50),
    }
}

pub(super) fn p95_completion_micros(
    evidence: &ActiveContextInput<'_>,
    candidate: &CandidateSnapshot,
    source: &str,
    existing: bool,
) -> u64 {
    let estimate = estimate(evidence, candidate, source, existing);
    p95_micros(
        &estimate,
        evidence.active.request().reserved_network_bytes(),
    )
}

fn estimate(
    evidence: &ActiveContextInput<'_>,
    candidate: &CandidateSnapshot,
    source: &str,
    existing: bool,
) -> OriginEstimate {
    let query = query(evidence, candidate, source, existing);
    evidence.inputs.stats.origin_model().estimate(
        &query,
        evidence.snapshot.observed_at_ms,
        DecisionMode::Normal,
    )
}

fn query(
    evidence: &ActiveContextInput<'_>,
    candidate: &CandidateSnapshot,
    source: &str,
    existing: bool,
) -> OriginQuery {
    let request = evidence.active.request();
    let context = OriginContext::new(
        method(request),
        request.reserved_network_bytes(),
        media(candidate),
    )
    .with_network(NetworkClass::Unavailable)
    .with_concurrency(concurrency(evidence, source, existing))
    .with_observed_at_ms(evidence.snapshot.observed_at_ms);
    OriginQuery::new(source, context)
}

fn concurrency(evidence: &ActiveContextInput<'_>, source: &str, existing: bool) -> usize {
    let Some(authority) = RequestAuthority::from_url(source) else {
        return 1;
    };
    let bodies = evidence.inputs.in_flight.iter().map(|item| item.identity());
    let probes = evidence.inputs.active_head_probes.iter();
    let active = bodies
        .chain(probes)
        .filter(|item| {
            RequestAuthority::from_url(item.source().as_str()) == Some(authority.clone())
        })
        .count();
    match existing {
        true => active.max(1),
        false => active.saturating_add(1),
    }
}

fn method(request: RetrievalRequest) -> RequestMethod {
    match request {
        RetrievalRequest::FetchRange { .. } => RequestMethod::RangeGet,
        RetrievalRequest::FetchWhole { .. } => RequestMethod::FullGet,
    }
}

fn media(candidate: &CandidateSnapshot) -> MediaClass {
    match candidate.layout {
        ghostr_engine::adaptive::MediaLayout::Unknown => MediaClass::Unknown,
        ghostr_engine::adaptive::MediaLayout::Streamable => MediaClass::ProgressiveMp4,
        ghostr_engine::adaptive::MediaLayout::RequiresCompleteFile => MediaClass::WholeObject,
    }
}

fn p95_micros(estimate: &OriginEstimate, bytes: u64) -> u64 {
    estimate
        .ttfb_ms
        .p95
        .saturating_mul(1_000)
        .saturating_add(transfer_micros(bytes, estimate.throughput_bps.p10))
}

fn transfer_micros(bytes: u64, throughput_bps: u64) -> u64 {
    u128::from(bytes)
        .saturating_mul(8 * MICROS_PER_SECOND)
        .checked_div(u128::from(throughput_bps.max(1)))
        .unwrap_or(u128::MAX)
        .min(u128::from(u64::MAX)) as u64
}

fn micros_to_ms(value: u64) -> u64 {
    value.saturating_add(999) / 1_000
}
