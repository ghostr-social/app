use crate::delivery_events::DecisionResolution;
use crate::evaluation::{IntegrityMetricEvent, TransferMetricEvent};
use crate::manager::completion_decision;
use crate::manager::inflight::{CompletionStatus, FinishedAction};
use crate::manager::transfers::ChunkDone;
use crate::manager::DeliveryWorker;
use ghostr_engine::adaptive::ResourceCost;

mod adaptation;
#[cfg(test)]
#[path = "completion_observability/completion_observability_axiom_test.rs"]
pub(crate) mod axiom_test_support;
mod outcome;
mod policy_limit;
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
        if let Some(event) = adaptation::event(
            self.keeper.stats().origin_model(),
            done,
            finished.exploration_admitted(),
        ) {
            evaluation.adaptation(&event);
        }
        if let Some(event) = integrity_event(done) {
            evaluation.integrity(event);
        }
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
