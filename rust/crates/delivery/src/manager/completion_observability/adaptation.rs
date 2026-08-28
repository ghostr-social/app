use super::outcome::result_bytes;
use crate::evaluation::AdaptationMetricEvent;
use crate::manager::transfers::ChunkDone;
use ghostr_engine::host_stats::host_of;
use ghostr_engine::origin_model::{
    AdaptationState, DecisionMode, OriginEstimate, OriginModel, OriginObservation, OriginOutcome,
};

pub(super) fn event(
    model: &OriginModel,
    done: &ChunkDone,
    exploration_admitted: bool,
) -> Option<AdaptationMetricEvent> {
    let observation = done.origin.as_ref()?;
    let estimate = model.estimate(
        &observation.query,
        observation.observed_at_ms,
        DecisionMode::Normal,
    );
    Some(metric(
        observation,
        estimate,
        exploration_admitted,
        result_bytes(done),
    ))
}

fn metric(
    observation: &OriginObservation,
    estimate: OriginEstimate,
    exploring: bool,
    bytes: u64,
) -> AdaptationMetricEvent {
    let failed = matches!(observation.outcome, OriginOutcome::Failure(_));
    let (exploration_bytes, failed_exploration_bytes) = exploration_cost(exploring, failed, bytes);
    AdaptationMetricEvent {
        origin: host_of(observation.query.url()).unwrap_or_else(|| "unavailable".into()),
        observed_at_ms: observation.observed_at_ms,
        adapting: estimate.adaptation == AdaptationState::Short,
        predicted_success_bps: probability_bps(estimate.success.mean),
        succeeded: success_observation(observation.outcome),
        latency_quantiles_on_time: latency_quantiles(observation, &estimate),
        regret_micros: regret_micros(observation, &estimate),
        exploration_bytes,
        failed_exploration_bytes,
    }
}

pub(super) fn exploration_cost(exploring: bool, failed: bool, bytes: u64) -> (u64, u64) {
    if !exploring {
        return (0, 0);
    }
    (bytes, if failed { bytes } else { 0 })
}

fn latency_quantiles(
    observation: &OriginObservation,
    estimate: &OriginEstimate,
) -> Option<[bool; 3]> {
    observation.ttfb_ms.map(|actual| {
        [
            actual <= estimate.ttfb_ms.p50,
            actual <= estimate.ttfb_ms.p95,
            actual <= estimate.ttfb_ms.p99,
        ]
    })
}

fn regret_micros(observation: &OriginObservation, estimate: &OriginEstimate) -> u64 {
    observation
        .ttfb_ms
        .map_or(0, |actual| actual.saturating_sub(estimate.ttfb_ms.p50))
        .saturating_mul(1_000)
}

fn success_observation(outcome: OriginOutcome) -> Option<bool> {
    match outcome {
        OriginOutcome::Success => Some(true),
        OriginOutcome::Failure(_) => Some(false),
        OriginOutcome::Cancelled => None,
    }
}

fn probability_bps(value: f64) -> u16 {
    (value.clamp(0.0, 1.0) * 10_000.0).round() as u16
}
