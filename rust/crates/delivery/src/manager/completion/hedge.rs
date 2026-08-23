use crate::manager::inflight::CompletionStatus;
use crate::manager::pressure::is_store_pressure;
use crate::manager::transfers::ChunkDone;
use crate::manager::DeliveryWorker;
use ghostr_engine::representation::TransferIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CompletionUse {
    Useful,
    OriginEvidence,
    Discarded,
}

pub(super) fn completion_use(status: CompletionStatus, done: &ChunkDone) -> CompletionUse {
    match status {
        CompletionStatus::Current | CompletionStatus::HedgeWinner => CompletionUse::Useful,
        CompletionStatus::HedgeLoser if !cancelled(done) => CompletionUse::OriginEvidence,
        CompletionStatus::Cancelled if policy_stop(done) => CompletionUse::OriginEvidence,
        CompletionStatus::Cancelled
        | CompletionStatus::HedgeLoser
        | CompletionStatus::Superseded => CompletionUse::Discarded,
    }
}

pub(super) fn record_origin_only(
    worker: &mut DeliveryWorker,
    done: &ChunkDone,
    identity: &TransferIdentity,
) {
    if done.outcome.as_ref().err().is_some_and(is_store_pressure) {
        return;
    }
    worker.keeper.note_chunk(done);
    match &done.outcome {
        Ok(_) => worker.note_successful_attempt(identity.post(), identity.source().as_str()),
        Err(error) => worker.absorb_failure(identity.post(), &done.url, error),
    }
}

fn cancelled(done: &ChunkDone) -> bool {
    done.outcome.as_ref().is_ok_and(|result| result.cancelled)
}

fn policy_stop(done: &ChunkDone) -> bool {
    done.outcome
        .as_ref()
        .err()
        .is_some_and(crate::chunk::whole_body_policy::is)
}

#[cfg(test)]
#[path = "cancelled_hedge_loser_evidence_test.rs"]
mod cancelled_hedge_loser_evidence_test;
#[cfg(test)]
#[path = "failed_hedge_loser_evidence_test.rs"]
mod failed_hedge_loser_evidence_test;
