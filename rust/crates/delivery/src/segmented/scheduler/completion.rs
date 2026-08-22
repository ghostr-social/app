use super::progress::Advance;
use super::{Active, SegmentedDelivery, SegmentedDone, SegmentedFinish};
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
        let source_index = active.pending.source_index;
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
        let finish = terminal(TerminalInput {
            context,
            result: &result,
            resources,
        });
        if let Some(error) = result
            .as_ref()
            .err()
            .filter(|error| !error.is_cancelled() && !error.is_superseded())
        {
            self.retry_or_fail(&post, generation, source_index, error.reason());
        }
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
        let advance = active.pending.advance(&object).map_err(|error| {
            FetchFailure::admitted(error, ErrorReason::InvalidResponse, telemetry, bytes)
        })?;
        self.cache
            .store_stage_object(post, generation, PreparedObject::from(object))
            .ok_or_else(|| FetchFailure::superseded(telemetry, bytes))?;
        match advance {
            Advance::Pending(next) => {
                self.pending.insert(post.clone(), next);
            }
            Advance::Ready => {
                self.cache.mark_stage_ready(post, generation);
            }
        }
        Ok(CompletedObject { bytes, telemetry })
    }

    fn retry_or_fail(
        &mut self,
        post: &ghostr_engine::PostId,
        generation: u64,
        source_index: usize,
        reason: ErrorReason,
    ) {
        let next = source_index.saturating_add(1);
        let source = self
            .targets
            .iter()
            .find(|target| &target.post == post)
            .and_then(|target| target.sources.get(next))
            .cloned();
        if let Some(source) = source {
            self.cache.reset_stage_retry(post, generation);
            self.pending.insert(
                post.clone(),
                super::progress::Pending::root(generation, next, source),
            );
        } else {
            self.cache
                .mark_stage_failed(post, generation, failure_detail(reason));
        }
    }
}

fn cancelled(outcome: Result<FetchedObject, FetchFailure>) -> FetchFailure {
    match outcome {
        Ok(object) => FetchFailure::cancelled(Some(object.telemetry), object.body.len() as u64),
        Err(failure) if failure.is_cancelled() => failure,
        Err(failure) => FetchFailure::cancelled(failure.origin(), failure.network_bytes()),
    }
}

fn failure_detail(reason: ErrorReason) -> String {
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
