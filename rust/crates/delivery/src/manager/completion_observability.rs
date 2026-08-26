use crate::delivery_events::DecisionResolution;
use crate::evaluation::{AdaptationMetricEvent, IntegrityMetricEvent, TransferMetricEvent};
use crate::manager::completion_decision;
use crate::manager::inflight::{CompletionStatus, FinishedAction};
use crate::manager::transfers::ChunkDone;
use crate::manager::DeliveryWorker;
use ghostr_engine::adaptive::ResourceCost;
use ghostr_engine::host_stats::host_of;
use ghostr_engine::origin_model::{AdaptationState, DecisionMode, OriginOutcome};

mod outcome;
mod policy_limit;
#[cfg(test)]
pub(crate) mod tests;
use outcome::{decision_outcome, result_bytes};

impl DeliveryWorker {
    pub(super) fn observe_chunk_completion(&mut self, done: &ChunkDone, finished: &FinishedAction) {
        let observed_at_ms = completion_time(done);
        let status = finished.status();
        if let Some(reservation) = finished.network_reservation() {
            self.warp_planner.reconcile_network_reservation(
                reservation.committed_bytes(),
                reservation.actual_bytes(done.received_bytes),
                observed_at_ms,
            );
        }
        let outcome = decision_outcome(status, done);
        let resolution = self.commands.resolve_decision_with_resources(
            done.attempt.id(),
            outcome,
            actual_resources(done),
            observed_at_ms,
        );
        let evaluation = self.commands.evaluation();
        evaluation.transfer(transfer_event(done, status, resolution.as_ref()));
        if let Some(event) = self.adaptation_event(done) {
            evaluation.adaptation(&event);
        }
        if let Some(event) = integrity_event(done) {
            evaluation.integrity(event);
        }
    }

    fn adaptation_event(&self, done: &ChunkDone) -> Option<AdaptationMetricEvent> {
        let observation = done.origin.as_ref()?;
        let estimate = self.keeper.stats().origin_model().estimate(
            &observation.query,
            observation.observed_at_ms,
            DecisionMode::Normal,
        );
        let bytes = result_bytes(done);
        let exploring = estimate.effective_samples < 8.0;
        let failed = matches!(observation.outcome, OriginOutcome::Failure(_));
        let regret_ms = observation
            .ttfb_ms
            .map_or(0, |actual| actual.saturating_sub(estimate.ttfb_ms.p50));
        Some(AdaptationMetricEvent {
            origin: host_of(observation.query.url()).unwrap_or_else(|| "unavailable".into()),
            observed_at_ms: observation.observed_at_ms,
            adapting: estimate.adaptation == AdaptationState::Short,
            predicted_success_bps: probability_bps(estimate.success.mean),
            succeeded: success_observation(observation.outcome),
            latency_quantiles_on_time: observation.ttfb_ms.map(|actual| {
                [
                    actual <= estimate.ttfb_ms.p50,
                    actual <= estimate.ttfb_ms.p95,
                    actual <= estimate.ttfb_ms.p99,
                ]
            }),
            regret_micros: regret_ms.saturating_mul(1_000),
            exploration_bytes: if exploring { bytes } else { 0 },
            failed_exploration_bytes: if exploring && failed { bytes } else { 0 },
        })
    }
}

fn completion_time(done: &ChunkDone) -> u64 {
    done.origin
        .as_ref()
        .map_or_else(crate::manager::time::unix_time_ms, |item| {
            item.observed_at_ms
        })
}

fn actual_resources(done: &ChunkDone) -> ResourceCost {
    ResourceCost::new(
        done.received_bytes,
        policy_limit::stored_bytes(&done.outcome),
        0,
        u16::from(done.request_started),
    )
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

fn transfer_event(
    done: &ChunkDone,
    status: CompletionStatus,
    resolution: Option<&DecisionResolution>,
) -> TransferMetricEvent {
    let bytes = result_bytes(done);
    let stored_bytes = policy_limit::stored_bytes(&done.outcome);
    let policy_limit = done
        .outcome
        .as_ref()
        .err()
        .is_some_and(|error| crate::chunk::whole_body_limit::from_error(error).is_some());
    let cancelled = status == CompletionStatus::Cancelled
        || done.outcome.as_ref().is_ok_and(|result| result.cancelled)
        || policy_limit;
    let result = done.outcome.as_ref().ok();
    let request_started = done.request_started;
    TransferMetricEvent {
        post: Some(done.attempt.chunk.post.clone()),
        total_bytes: bytes,
        aborted_bytes: if cancelled { bytes } else { 0 },
        duplicate_hedge_bytes: if status == CompletionStatus::HedgeLoser {
            bytes
        } else {
            0
        },
        completable_probe_bytes: completable_probe_bytes(done, resolution, bytes),
        full_download_started: request_started
            && resolution.is_some_and(completion_decision::is_whole),
        request_started,
        promotion_avoided_restart: result.is_some_and(|item| item.promoted),
        storage_byte_ms: byte_millis(stored_bytes, resolution.map_or(0, |item| item.elapsed_ms)),
        ..TransferMetricEvent::default()
    }
}

fn completable_probe_bytes(
    done: &ChunkDone,
    resolution: Option<&DecisionResolution>,
    bytes: u64,
) -> u64 {
    const LIMIT: u64 = 1_048_576;
    let Some(resolution) = resolution else {
        return 0;
    };
    let probe = completion_decision::is_probe(resolution);
    let small = done
        .outcome
        .as_ref()
        .ok()
        .and_then(|result| result.total_bytes)
        .is_some_and(|total| total <= LIMIT);
    if probe && small {
        bytes
    } else {
        0
    }
}

fn integrity_event(done: &ChunkDone) -> Option<IntegrityMetricEvent> {
    let error = done.outcome.as_ref().err()?;
    if error
        .downcast_ref::<ghostr_partial_store::partial_range_completion::IntegrityMismatch>()
        .is_some()
    {
        return Some(IntegrityMetricEvent::HashMismatch);
    }
    let message = format!("{error:#}").to_ascii_lowercase();
    (message.contains("redirect target is not public")
        || message.contains("destination is not public"))
    .then_some(IntegrityMetricEvent::SsrfOrRedirectBlock)
}

fn byte_millis(bytes: u64, elapsed_ms: u64) -> u64 {
    u128::from(bytes)
        .saturating_mul(u128::from(elapsed_ms))
        .min(u128::from(u64::MAX)) as u64
}
