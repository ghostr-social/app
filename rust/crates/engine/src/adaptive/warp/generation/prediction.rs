use super::super::{ActionForecast, ActionKind};
use crate::adaptive::{
    AllocationPlan, CandidateSnapshot, CompletionTimes, ControlMode, PlayabilitySnapshot,
};
use crate::origin_model::{DecisionMode, OriginModel, OriginQuery, OriginRequestProfile};

mod readiness;

#[derive(Clone, Copy)]
pub(super) struct Prediction {
    pub forecast: ActionForecast,
    pub uncertainty_bps: u16,
    pub request_profile: Option<OriginRequestProfile>,
}

#[derive(Clone, Copy)]
pub(super) struct PredictionInput<'a> {
    pub model: &'a OriginModel,
    pub snapshot: &'a PlayabilitySnapshot,
    pub base: &'a AllocationPlan,
    pub candidate: &'a CandidateSnapshot,
    pub action: &'a ActionKind,
    pub source: &'a str,
    pub concurrency: usize,
    pub mode: ControlMode,
    pub direct_playback_blocked: bool,
    pub network_class: crate::origin_model::NetworkClass,
}

pub(super) fn predict(input: PredictionInput<'_>) -> Prediction {
    let request_profile = super::request_profile::for_action(input.candidate, input.action);
    let profile = request_profile.expect("network predictions require a request profile");
    let bytes = profile.planned_bytes();
    let query = OriginQuery::new(
        input.source,
        profile
            .context()
            .with_concurrency(input.concurrency)
            .with_network(input.network_class)
            .with_observed_at_ms(input.snapshot.observed_at_ms),
    );
    let estimate = input.model.estimate(
        &query,
        input.snapshot.observed_at_ms,
        decision_mode(input.mode),
    );
    let success = range_adjusted_success(input.action, &estimate);
    Prediction {
        forecast: ActionForecast::new(
            completion(bytes, &estimate),
            basis_points(success),
            ready_gain(
                input.candidate,
                input.action,
                input.base,
                input.direct_playback_blocked,
            ),
        ),
        uncertainty_bps: basis_points(estimate.uncertainty),
        request_profile,
    }
}

pub(super) fn estimate_open_body(
    input: PredictionInput<'_>,
    profile: OriginRequestProfile,
) -> crate::origin_model::OriginEstimate {
    let query = OriginQuery::new(
        input.source,
        profile
            .context()
            .with_concurrency(input.concurrency)
            .with_network(input.network_class)
            .with_observed_at_ms(input.snapshot.observed_at_ms),
    );
    input.model.estimate_open_body(
        &query,
        input.snapshot.observed_at_ms,
        decision_mode(input.mode),
    )
}

pub(super) fn transform_prediction(candidate: &CandidateSnapshot, cpu_ms: u64) -> Prediction {
    Prediction {
        forecast: ActionForecast::new(
            CompletionTimes::new(cpu_ms, cpu_ms, cpu_ms, cpu_ms),
            10_000,
            candidate.duration_ms,
        ),
        uncertainty_bps: 0,
        request_profile: None,
    }
}

pub(super) fn completion(
    bytes: u64,
    estimate: &crate::origin_model::OriginEstimate,
) -> CompletionTimes {
    let expected = estimate
        .ttfb_ms
        .p50
        .saturating_add(transfer_ms(bytes, estimate.throughput_bps.p50));
    let p95 = estimate
        .ttfb_ms
        .p95
        .saturating_add(transfer_ms(bytes, estimate.throughput_bps.p10));
    let tail_rate = estimate
        .throughput_bps
        .p10
        .min(estimate.throughput_bps.selected)
        .max(1);
    let p99 = estimate
        .ttfb_ms
        .p99
        .saturating_add(transfer_ms(bytes, tail_rate));
    CompletionTimes::new(expected, p95.max(expected), p99.max(p95), cvar(p95, p99))
}

pub(super) fn cvar(p95: u64, p99: u64) -> u64 {
    p99.saturating_add(p99.saturating_sub(p95) / 2)
}

pub(super) fn transfer_ms(bytes: u64, throughput_bps: u64) -> u64 {
    bytes.saturating_mul(8_000) / throughput_bps.max(1)
}

fn range_adjusted_success(
    action: &ActionKind,
    estimate: &crate::origin_model::OriginEstimate,
) -> f64 {
    let range = match action {
        ActionKind::Prefix(_) | ActionKind::Tail(_) | ActionKind::FetchRange(_) => estimate
            .range_compliance
            .map_or(1.0, |value| value.selected),
        _ => 1.0,
    };
    estimate.success.selected * range
}

pub(super) fn ready_gain(
    candidate: &CandidateSnapshot,
    action: &ActionKind,
    base: &AllocationPlan,
    direct_playback_blocked: bool,
) -> u64 {
    if direct_playback_blocked {
        return 0;
    }
    readiness::gain(candidate, action, base)
}

pub(super) fn decision_mode(mode: ControlMode) -> DecisionMode {
    match mode {
        ControlMode::Emergency => DecisionMode::Emergency,
        ControlMode::Safety => DecisionMode::Safety,
        ControlMode::Normal => DecisionMode::Normal,
    }
}

pub(super) fn basis_points(value: f64) -> u16 {
    (value.clamp(0.0, 1.0) * 10_000.0).round() as u16
}
