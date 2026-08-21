use super::super::{ActionForecast, ActionKind};
use crate::adaptive::{
    CandidateSnapshot, CompletionTimes, ControlMode, MediaLayout, PlayabilitySnapshot,
};
use crate::origin_model::{
    DecisionMode, MediaClass, OriginContext, OriginModel, OriginQuery, RequestMethod,
};

#[derive(Clone, Copy)]
pub(super) struct Prediction {
    pub forecast: ActionForecast,
    pub uncertainty_bps: u16,
}

pub(super) struct PredictionInput<'a> {
    pub model: &'a OriginModel,
    pub snapshot: &'a PlayabilitySnapshot,
    pub candidate: &'a CandidateSnapshot,
    pub action: &'a ActionKind,
    pub source: &'a str,
    pub concurrency: usize,
    pub mode: ControlMode,
}

pub(super) fn predict(input: PredictionInput<'_>) -> Prediction {
    let bytes = action_bytes(input.action);
    let query = OriginQuery::new(
        input.source,
        OriginContext::new(
            method(input.action),
            bytes,
            media(input.candidate, input.action),
        )
        .with_concurrency(input.concurrency)
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
            ready_gain(input.candidate, input.action),
        ),
        uncertainty_bps: basis_points(estimate.uncertainty),
    }
}

fn completion(bytes: u64, estimate: &crate::origin_model::OriginEstimate) -> CompletionTimes {
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

fn cvar(p95: u64, p99: u64) -> u64 {
    p99.saturating_add(p99.saturating_sub(p95) / 2)
}

fn transfer_ms(bytes: u64, throughput_bps: u64) -> u64 {
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

fn action_bytes(action: &ActionKind) -> u64 {
    match action {
        ActionKind::Prefix(range)
        | ActionKind::Tail(range)
        | ActionKind::FetchRange(range)
        | ActionKind::CacheUpgrade(range) => range.len(),
        ActionKind::FetchWhole { maximum_bytes } => *maximum_bytes,
        _ => 0,
    }
}

fn ready_gain(candidate: &CandidateSnapshot, action: &ActionKind) -> u64 {
    match action {
        ActionKind::FetchWhole { .. } => candidate.duration_ms,
        ActionKind::Prefix(range) | ActionKind::FetchRange(range) => candidate
            .playable_ranges
            .iter()
            .filter(|item| overlaps(item.bytes, *range))
            .map(|item| item.playable_ms)
            .sum(),
        _ => 0,
    }
}

fn overlaps(left: crate::ByteRange, right: crate::ByteRange) -> bool {
    left.start < right.end && right.start < left.end
}

fn method(action: &ActionKind) -> RequestMethod {
    match action {
        ActionKind::Head => RequestMethod::Head,
        ActionKind::Prefix(_) => RequestMethod::PrefixGet,
        ActionKind::Tail(_) => RequestMethod::TailGet,
        ActionKind::FetchRange(_) | ActionKind::CacheUpgrade(_) | ActionKind::Hedge { .. } => {
            RequestMethod::RangeGet
        }
        ActionKind::FetchWhole { .. } | ActionKind::Promote { .. } => RequestMethod::FullGet,
        ActionKind::Transform(_) | ActionKind::Cancel(_) => RequestMethod::RangeGet,
    }
}

fn media(candidate: &CandidateSnapshot, action: &ActionKind) -> MediaClass {
    if matches!(action, ActionKind::Transform(_)) {
        return MediaClass::TransformRequired;
    }
    match candidate.layout {
        MediaLayout::Unknown => MediaClass::Unknown,
        MediaLayout::Streamable => MediaClass::ProgressiveMp4,
        MediaLayout::RequiresCompleteFile => MediaClass::WholeObject,
    }
}

fn decision_mode(mode: ControlMode) -> DecisionMode {
    match mode {
        ControlMode::Emergency => DecisionMode::Emergency,
        ControlMode::Safety => DecisionMode::Safety,
        ControlMode::Normal => DecisionMode::Normal,
    }
}

fn basis_points(value: f64) -> u16 {
    (value.clamp(0.0, 1.0) * 10_000.0).round() as u16
}
