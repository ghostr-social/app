use super::progress::Advance;
use super::{Active, SegmentedDelivery, SegmentedDone, SegmentedFinish};
use crate::segmented::cache::{CompleteStage, StageBlock, StoredStage};
use crate::segmented::fetch::{FetchFailure, FetchedObject, OriginTelemetry};
use crate::segmented::prepare::PreparedObject;
use ghostr_engine::origin_model::ErrorReason;

mod terminal;
use terminal::{terminal, TerminalContext, TerminalInput};

pub(super) struct CompletedObject {
    pub(super) bytes: u64,
    pub(super) telemetry: OriginTelemetry,
}

impl SegmentedDelivery {
    pub(crate) fn finish(&mut self, done: SegmentedDone) -> Option<SegmentedFinish> {
        let current = self.active.get(&done.post).is_some_and(|active| {
            active.action == done.action && active.pending.generation == done.generation
        });
        if !current {
            return None;
        }
        let active = self.active.remove(&done.post)?;
        let stage = active.pending.stage;
        let source = active.pending.url.clone();
        let action = done.action;
        let post = done.post;
        let generation = done.generation;
        let observed_at_ms = done.observed_at_ms;
        let resources = done.resources;
        let outcome = match active.cancelling {
            true => Err(cancelled(done.outcome)),
            false => done.outcome,
        };
        let result = self.complete_stage(&post, generation, &active, outcome);
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
        generation: u64,
        active: &Active,
        outcome: Result<FetchedObject, FetchFailure>,
    ) -> Result<CompletedObject, FetchFailure> {
        match outcome {
            Err(failure) => Err(failure),
            Ok(object) => self.store_completed(post, generation, active, object),
        }
    }

    fn store_completed(
        &mut self,
        post: &ghostr_engine::PostId,
        generation: u64,
        active: &Active,
        object: FetchedObject,
    ) -> Result<CompletedObject, FetchFailure> {
        let bytes = object.body.len() as u64;
        let telemetry = object.telemetry;
        let completed = CompletedObject { bytes, telemetry };
        let offset = object.offset;
        let continuation = object.continuation.clone();
        let stored = self
            .cache
            .store_stage_block(
                post,
                generation,
                StageBlock::new(offset, PreparedObject::from(object), continuation.is_none()),
            )
            .ok_or_else(|| FetchFailure::superseded(telemetry, bytes))?;
        match stored {
            StoredStage::Partial => self.continue_stage(post, active, continuation),
            StoredStage::Complete(object) => {
                self.advance_stage(post, active, *object, &completed)?;
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
        object: CompleteStage,
        completed: &CompletedObject,
    ) -> Result<(), FetchFailure> {
        let advance = active.pending.advance(&object.object).map_err(|error| {
            FetchFailure::admitted(
                error,
                ErrorReason::InvalidResponse,
                completed.telemetry,
                completed.bytes,
            )
        })?;
        if !self
            .cache
            .commit_stage_complete(post, active.pending.generation, object)
        {
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
                self.cache.mark_stage_ready(post, active.pending.generation);
            }
        }
        Ok(())
    }
}

fn cancelled(outcome: Result<FetchedObject, FetchFailure>) -> FetchFailure {
    match outcome {
        Ok(object) => FetchFailure::cancelled(Some(object.telemetry), object.body.len() as u64),
        Err(failure) if failure.is_cancelled() => failure,
        Err(failure) => FetchFailure::cancelled(failure.origin(), failure.network_bytes()),
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
