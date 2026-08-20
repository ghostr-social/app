use super::super::PlanInputs;
use ghostr_engine::adaptive::{
    ControlMode, DecisionModelInput, MediaLayout, PlayabilitySnapshot, ShadowPrices,
};
use ghostr_engine::origin_model::{
    AdaptationState, DecisionMode, MediaClass, OriginContext, OriginQuery, RequestMethod,
};

pub(super) fn models(
    snapshot: &PlayabilitySnapshot,
    inputs: &PlanInputs<'_>,
    mode: ControlMode,
) -> Vec<DecisionModelInput> {
    snapshot
        .candidates
        .iter()
        .flat_map(|candidate| {
            candidate.origins.iter().map(move |origin| {
                let query = query(candidate, &origin.source, snapshot.observed_at_ms);
                let estimate = inputs.stats.origin_model().estimate(
                    &query,
                    snapshot.observed_at_ms,
                    decision_mode(mode),
                );
                DecisionModelInput {
                    source: origin.source.clone(),
                    success_bps: probability(estimate.success),
                    range_compliance_bps: estimate
                        .range_compliance
                        .map_or([10_000; 4], probability),
                    ttfb_ms: quantiles(estimate.ttfb_ms),
                    throughput_bps: quantiles(estimate.throughput_bps),
                    effective_samples: estimate.effective_samples.round() as u64,
                    adapting: estimate.adaptation == AdaptationState::Short,
                    uncertainty_bps: basis_points(estimate.uncertainty),
                }
            })
        })
        .take(64)
        .collect()
}

pub(super) fn shadow_prices(snapshot: &PlayabilitySnapshot) -> ShadowPrices {
    let storage = fraction(snapshot.storage.used_bytes, snapshot.storage.budget_bytes);
    let active = snapshot
        .candidates
        .iter()
        .map(|candidate| candidate.in_flight.len() as u64)
        .sum();
    let connections = snapshot.network.connection_ceiling.max(1) as u64;
    let requests = fraction(active, connections);
    let network = requests.saturating_add(u64::from(snapshot.network.packet_loss_bps) * 100);
    ShadowPrices::new(network, storage, 0, requests)
}

fn query(
    candidate: &ghostr_engine::adaptive::CandidateSnapshot,
    source: &str,
    observed_at_ms: u64,
) -> OriginQuery {
    let method = match candidate.layout {
        MediaLayout::RequiresCompleteFile => RequestMethod::FullGet,
        _ => RequestMethod::RangeGet,
    };
    let bytes = candidate
        .total_bytes
        .unwrap_or(ghostr_engine::adaptive::REQUEST_SLICE_BYTES);
    OriginQuery::new(
        source,
        OriginContext::new(method, bytes, MediaClass::ProgressiveMp4)
            .with_observed_at_ms(observed_at_ms),
    )
}

fn decision_mode(mode: ControlMode) -> DecisionMode {
    match mode {
        ControlMode::Normal => DecisionMode::Normal,
        ControlMode::Safety => DecisionMode::Safety,
        ControlMode::Emergency => DecisionMode::Emergency,
    }
}

fn probability(value: ghostr_engine::origin_model::ProbabilityEstimate) -> [u16; 4] {
    [
        basis_points(value.mean),
        basis_points(value.lower),
        basis_points(value.upper),
        basis_points(value.selected),
    ]
}

fn quantiles(value: ghostr_engine::origin_model::QuantileEstimate) -> [u64; 6] {
    [
        value.p10,
        value.p50,
        value.p90,
        value.p95,
        value.p99,
        value.selected,
    ]
}

fn basis_points(value: f64) -> u16 {
    (value.clamp(0.0, 1.0) * 10_000.0).round() as u16
}

fn fraction(value: u64, capacity: u64) -> u64 {
    u128::from(value)
        .saturating_mul(1_000_000)
        .checked_div(u128::from(capacity.max(1)))
        .unwrap_or_default()
        .min(1_000_000) as u64
}
