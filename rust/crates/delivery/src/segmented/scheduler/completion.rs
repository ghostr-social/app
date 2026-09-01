use super::prepared::{PreparedStage, PreparedTransfer};
use super::progress::Advance;
use super::{Active, SegmentedDelivery, SegmentedDone, SegmentedFinish};
use crate::segmented::cache::StageLease;
use crate::segmented::fetch::{FetchFailure, OriginTelemetry};
use crate::segmented::prepare::PreparedComplete;
use ghostr_engine::origin_model::ErrorReason;

mod terminal;
use terminal::{terminal, TerminalContext, TerminalInput};

#[cfg(test)]
#[path = "completion/cancellation_conversion_test.rs"]
mod cancellation_conversion_test;

pub(super) struct CompletedObject {
    bytes: u64,
    telemetry: OriginTelemetry,
}

struct LeasedComplete {
    object: PreparedComplete,
    lease: StageLease,
}

impl SegmentedDelivery {
    pub(crate) fn finish(&mut self, done: SegmentedDone) -> Option<SegmentedFinish> {
        let current = self
            .active
            .get(&done.post)
            .is_some_and(|active| active.action == done.action && active.fence == done.fence);
        if !current {
            return None;
        }
        let active = self.active.remove(&done.post)?;
        let stage = active.pending.stage;
        let source = active.pending.url.clone();
        let action = done.action;
        let post = done.post;
        let observed_at_ms = done.observed_at_ms;
        let resources = done.resources;
        let outcome = if active.cancelling {
            Err(cancelled(done.outcome))
        } else {
            done.outcome
        };
        let result = self.complete_stage(&post, &active, outcome);
        let context = TerminalContext::new(&source, stage, action, observed_at_ms);
        let mut finish = terminal(TerminalInput {
            context,
            result: &result,
            resources,
        });
        finish.recovery = self.recovery(&post, &active.pending, &result);
        Some(finish)
    }

    fn complete_stage(
        &mut self,
        post: &ghostr_engine::PostId,
        active: &Active,
        outcome: Result<PreparedTransfer, FetchFailure>,
    ) -> Result<CompletedObject, FetchFailure> {
        match outcome {
            Err(failure) => Err(failure),
            Ok(object) => self.store_completed(post, active, object),
        }
    }

    fn store_completed(
        &mut self,
        post: &ghostr_engine::PostId,
        active: &Active,
        transfer: PreparedTransfer,
    ) -> Result<CompletedObject, FetchFailure> {
        let PreparedTransfer {
            received_bytes: bytes,
            telemetry,
            continuation,
            stage,
            lease,
        } = transfer;
        let completed = CompletedObject { bytes, telemetry };
        match stage {
            PreparedStage::Partial(object) => {
                if !lease.commit_partial(object) {
                    return Err(FetchFailure::superseded(telemetry, bytes));
                }
                self.continue_stage(post, active, continuation);
            }
            PreparedStage::Complete(object) => {
                self.advance_stage(post, active, LeasedComplete { object, lease }, &completed)?;
            }
        }
        Ok(completed)
    }

    fn continue_stage(
        &mut self,
        post: &ghostr_engine::PostId,
        active: &Active,
        continuation: Option<crate::segmented::fetch::ObjectContinuation>,
    ) {
        let continuation = continuation.expect("partial HLS stage has a continuation");
        self.pending
            .insert(post.clone(), active.pending.continued(continuation));
    }

    fn advance_stage(
        &mut self,
        post: &ghostr_engine::PostId,
        active: &Active,
        stage: LeasedComplete,
        completed: &CompletedObject,
    ) -> Result<(), FetchFailure> {
        let advance = active
            .pending
            .advance(&stage.object.object)
            .map_err(|error| {
                FetchFailure::admitted(
                    error,
                    ErrorReason::InvalidResponse,
                    completed.telemetry,
                    completed.bytes,
                )
            })?;
        if !stage.lease.commit_complete(stage.object) {
            return Err(FetchFailure::superseded(
                completed.telemetry,
                completed.bytes,
            ));
        }
        match advance {
            Advance::Pending(next) => {
                let attempt = self.allocate_attempt();
                self.pending
                    .insert(post.clone(), (*next).with_attempt(attempt));
            }
            Advance::Ready => {
                self.cache.mark_stage_ready_for_playback(
                    post,
                    active.pending.generation,
                    &active.pending.playback_manifest,
                );
            }
        }
        Ok(())
    }
}

fn cancelled(outcome: Result<PreparedTransfer, FetchFailure>) -> FetchFailure {
    match outcome {
        Ok(transfer) => transfer.cancelled_failure(),
        Err(failure) if failure.is_cancelled() => failure,
        Err(failure) => failure.into_cancelled(),
    }
}

pub(super) fn failure_detail(reason: ErrorReason) -> String {
    match reason {
        ErrorReason::Timeout => "HLS bootstrap timed out",
        ErrorReason::Dns => "HLS bootstrap could not resolve the media host",
        ErrorReason::Tls => "HLS bootstrap could not establish a secure connection",
        ErrorReason::Http4xx => "HLS bootstrap request was rejected",
        ErrorReason::Http5xx => "HLS bootstrap origin is unavailable",
        ErrorReason::InvalidResponse | ErrorReason::RangeNoncompliant => {
            "HLS bootstrap received an invalid response"
        }
        ErrorReason::Connection => "HLS bootstrap connection failed",
        ErrorReason::Policy => "HLS bootstrap was blocked by media policy",
        ErrorReason::Unknown => "HLS bootstrap failed",
    }
    .to_owned()
}
