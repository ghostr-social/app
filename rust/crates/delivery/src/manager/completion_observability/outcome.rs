use super::policy_limit;
use crate::manager::failure::classify;
use crate::manager::inflight::CompletionStatus;
use crate::manager::transfers::ChunkDone;
use ghostr_engine::adaptive::DecisionOutcome;

pub(super) fn decision_outcome(status: CompletionStatus, done: &ChunkDone) -> DecisionOutcome {
    if status == CompletionStatus::Superseded {
        return DecisionOutcome::Superseded;
    }
    if let Err(error) = &done.outcome {
        if policy_limit::is(error) {
            return policy_limit::decision(error);
        }
    }
    if status == CompletionStatus::Cancelled {
        return cancelled(done);
    }
    match &done.outcome {
        Ok(result) if result.cancelled => cancelled(done),
        Ok(result) if result.range_ignored => DecisionOutcome::Failed {
            class: "warp_range_noncompliant".into(),
            elapsed_ms: 0,
        },
        Ok(_) => DecisionOutcome::Succeeded {
            bytes: done.received_bytes,
            elapsed_ms: 0,
        },
        Err(error) if crate::chunk::sink::is_local_store_failure(error) => {
            DecisionOutcome::Failed {
                class: "warp_local_store_failure".into(),
                elapsed_ms: 0,
            }
        }
        Err(error) => DecisionOutcome::Failed {
            class: format!("{:?}", classify(error)),
            elapsed_ms: 0,
        },
    }
}

fn cancelled(done: &ChunkDone) -> DecisionOutcome {
    DecisionOutcome::Cancelled {
        bytes: result_bytes(done),
        elapsed_ms: 0,
    }
}

pub(super) const fn result_bytes(done: &ChunkDone) -> u64 {
    done.received_bytes
}
