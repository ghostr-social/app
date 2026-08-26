use super::super::{ActionKind, ActionValue};
use super::prediction::Prediction;
use crate::adaptive::{CandidateSnapshot, ControlMode};

pub(super) fn score(
    candidate: &CandidateSnapshot,
    action: &ActionKind,
    prediction: Prediction,
    mode: ControlMode,
) -> ActionValue {
    let reach_bps = probability_bps(candidate.view_probability.value());
    let ready = prediction.forecast.ready_playback_ms;
    let success = u64::from(prediction.forecast.success_bps);
    let delay = scale(ready.saturating_mul(1_000), reach_bps);
    let reserve = scale(scale(ready.saturating_mul(600), reach_bps), success);
    let information = information_value(candidate, action, prediction.uncertainty_bps);
    let exploration = exploration_value(action, prediction.uncertainty_bps, mode);
    ActionValue {
        delay_loss_micros: as_i64(delay.saturating_mul(urgency(mode))),
        reserve_gain_micros: as_i64(reserve.saturating_mul(reserve_weight(mode))),
        information_value_micros: as_i64(information),
        exploration_micros: as_i64(exploration),
        cache_gain_micros: as_i64(cache_gain(candidate, action, reach_bps)),
        tail_risk_micros: as_i64(tail_risk(prediction, mode)),
        cvar_micros: as_i64(
            prediction
                .forecast
                .completion
                .cvar_ms
                .saturating_mul(cvar_weight(mode)),
        ),
        rank_cost_micros: as_i64(candidate.feed_offset.magnitude() as u64 * 25_000),
    }
}

fn information_value(
    candidate: &CandidateSnapshot,
    action: &ActionKind,
    uncertainty_bps: u16,
) -> u64 {
    let explicit_probe =
        candidate.total_bytes.is_none() && matches!(action, ActionKind::FetchWhole { .. });
    if !explicit_probe
        && !matches!(
            action,
            ActionKind::Head
                | ActionKind::Prefix(_)
                | ActionKind::Tail(_)
                | ActionKind::HlsBootstrap { .. }
        )
    {
        return 0;
    }
    let unresolved =
        candidate.evidence.missing.len() as u64 + candidate.evidence.conflicts.len() as u64 * 2;
    unresolved
        .saturating_add(1)
        .saturating_mul(u64::from(uncertainty_bps))
        .saturating_mul(100)
}

fn exploration_value(action: &ActionKind, uncertainty_bps: u16, mode: ControlMode) -> u64 {
    if mode != ControlMode::Normal || !low_cost_probe(action) {
        return 0;
    }
    u64::from(uncertainty_bps).saturating_mul(50)
}

fn low_cost_probe(action: &ActionKind) -> bool {
    match action {
        ActionKind::Head => true,
        ActionKind::Prefix(range) | ActionKind::Tail(range) => range.len() <= 65_536,
        ActionKind::HlsBootstrap { stage, .. } => stage.is_manifest(),
        _ => false,
    }
}

fn cache_gain(candidate: &CandidateSnapshot, action: &ActionKind, reach_bps: u64) -> u64 {
    if candidate.total_bytes.is_none() && matches!(action, ActionKind::FetchWhole { .. }) {
        return 0;
    }
    let bytes = match action {
        ActionKind::FetchWhole { maximum_bytes }
        | ActionKind::HlsBootstrap { maximum_bytes, .. } => *maximum_bytes,
        ActionKind::CacheUpgrade(range) => range.len(),
        _ => 0,
    };
    scale(bytes, reach_bps).saturating_mul(2)
        + candidate
            .present
            .iter()
            .map(|range| range.len())
            .sum::<u64>()
            / 8
}

fn tail_risk(prediction: Prediction, mode: ControlMode) -> u64 {
    prediction
        .forecast
        .completion
        .p99_ms
        .saturating_sub(prediction.forecast.completion.expected_ms)
        .saturating_mul(tail_weight(mode))
}

fn urgency(mode: ControlMode) -> u64 {
    match mode {
        ControlMode::Emergency => 4,
        ControlMode::Safety => 2,
        ControlMode::Normal => 1,
    }
}

fn reserve_weight(mode: ControlMode) -> u64 {
    match mode {
        ControlMode::Emergency => 3,
        ControlMode::Safety => 2,
        ControlMode::Normal => 1,
    }
}

fn tail_weight(mode: ControlMode) -> u64 {
    match mode {
        ControlMode::Emergency => 2_000,
        ControlMode::Safety => 1_000,
        ControlMode::Normal => 500,
    }
}

fn cvar_weight(mode: ControlMode) -> u64 {
    match mode {
        ControlMode::Emergency => 1_000,
        ControlMode::Safety => 500,
        ControlMode::Normal => 200,
    }
}

fn probability_bps(value: f64) -> u64 {
    (value.clamp(0.0, 1.0) * 10_000.0).round() as u64
}

fn scale(value: u64, basis_points: u64) -> u64 {
    value.saturating_mul(basis_points) / 10_000
}

fn as_i64(value: u64) -> i64 {
    value.min(i64::MAX as u64) as i64
}
