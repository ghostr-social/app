use super::CompletedObject;
use crate::segmented::fetch::{FetchFailure, OriginTelemetry};
use crate::segmented::scheduler::SegmentedFinish;
use crate::segmented::scheduler::SegmentedResourceCommitment;
use ghostr_engine::adaptive::{DecisionOutcome, HlsBootstrapStage, ResourceCost};
use ghostr_engine::origin_model::{
    MediaClass, OriginContext, OriginObservation, OriginQuery, RequestMethod,
};

const MIN_RELIABLE_THROUGHPUT_SAMPLE_BYTES: u64 = 65_536;

mod failure;
use failure::class as failure_class;

#[cfg(test)]
#[path = "terminal/cancellation_test.rs"]
mod cancellation_test;
#[cfg(test)]
#[path = "terminal/failure_class_test.rs"]
mod failure_class_test;
#[cfg(test)]
mod tests;
#[cfg(test)]
#[path = "terminal/throughput_test.rs"]
mod throughput_test;

#[derive(Clone, Copy)]
pub(super) struct TerminalContext<'a> {
    source: &'a str,
    stage: HlsBootstrapStage,
    action: ghostr_engine::ActionId,
    observed_at_ms: u64,
}

impl<'a> TerminalContext<'a> {
    pub(super) const fn new(
        source: &'a str,
        stage: HlsBootstrapStage,
        action: ghostr_engine::ActionId,
        observed_at_ms: u64,
    ) -> Self {
        Self {
            source,
            stage,
            action,
            observed_at_ms,
        }
    }
}

pub(super) struct TerminalInput<'a> {
    pub context: TerminalContext<'a>,
    pub result: &'a Result<CompletedObject, FetchFailure>,
    pub resources: SegmentedResourceCommitment,
}

pub(super) fn terminal(input: TerminalInput<'_>) -> SegmentedFinish {
    let telemetry = input
        .result
        .as_ref()
        .map(|completed| completed.telemetry)
        .ok()
        .or_else(|| {
            input
                .result
                .as_ref()
                .err()
                .and_then(|failure| failure.origin())
        });
    let (outcome, actual_resources) = decision_outcome(input.result, telemetry);
    SegmentedFinish {
        action: input.context.action,
        outcome,
        observation: telemetry.and_then(|timing| {
            observation(
                input.context,
                timing,
                input.result,
                input.resources.expected_network_bytes(),
            )
        }),
        actual_resources,
        resources: input.resources,
    }
}

fn decision_outcome(
    result: &Result<CompletedObject, FetchFailure>,
    telemetry: Option<OriginTelemetry>,
) -> (DecisionOutcome, Option<ResourceCost>) {
    match result {
        Ok(completed) => (
            DecisionOutcome::Succeeded {
                bytes: completed.bytes,
                elapsed_ms: duration_ms(completed.telemetry.elapsed),
            },
            Some(ResourceCost::new(completed.bytes, completed.bytes, 0, 1)),
        ),
        Err(failure) if failure.is_cancelled() => (
            DecisionOutcome::Cancelled {
                bytes: failure.network_bytes(),
                elapsed_ms: telemetry.map_or(0, |value| duration_ms(value.elapsed)),
            },
            failure.actual_resources(),
        ),
        Err(failure) if failure.is_superseded() => {
            (DecisionOutcome::Superseded, failure.actual_resources())
        }
        Err(failure) => (
            DecisionOutcome::Failed {
                class: failure_class(failure.reason()).to_owned(),
                elapsed_ms: telemetry.map_or(0, |value| duration_ms(value.elapsed)),
            },
            failure.actual_resources(),
        ),
    }
}

fn observation(
    context: TerminalContext<'_>,
    timing: OriginTelemetry,
    result: &Result<CompletedObject, FetchFailure>,
    expected_bytes: u64,
) -> Option<OriginObservation> {
    let query = OriginQuery::new(
        context.source,
        OriginContext::new(method(context.stage), expected_bytes, MediaClass::Segmented)
            .with_concurrency(timing.concurrency)
            .with_network(timing.network_class)
            .with_observed_at_ms(context.observed_at_ms),
    );
    let observation = match result {
        Ok(completed) => {
            success_observation(query, context.observed_at_ms, completed.bytes, timing)
        }
        Err(failure) if failure.is_superseded() => success_observation(
            query,
            context.observed_at_ms,
            failure.network_bytes(),
            timing,
        ),
        Err(failure)
            if failure.is_cancelled()
                || failure.reason() == ghostr_engine::origin_model::ErrorReason::Policy =>
        {
            return None;
        }
        Err(failure) => OriginObservation::failure(query, context.observed_at_ms, failure.reason()),
    };
    Some(observation)
}

fn success_observation(
    query: OriginQuery,
    observed_at_ms: u64,
    bytes: u64,
    timing: OriginTelemetry,
) -> OriginObservation {
    let mut success = OriginObservation::success(query, observed_at_ms)
        .with_ttfb_ms(duration_ms(timing.ttfb.unwrap_or(timing.elapsed)));
    success.throughput_bps = throughput(bytes, timing.elapsed, timing.ttfb);
    success
}

fn duration_ms(value: std::time::Duration) -> u64 {
    value.as_millis().min(u128::from(u64::MAX)).max(1) as u64
}

fn method(stage: HlsBootstrapStage) -> RequestMethod {
    match stage.is_manifest() {
        true => RequestMethod::ManifestGet,
        false => RequestMethod::SegmentGet,
    }
}

fn throughput(
    bytes: u64,
    elapsed: std::time::Duration,
    ttfb: Option<std::time::Duration>,
) -> Option<u64> {
    if bytes < MIN_RELIABLE_THROUGHPUT_SAMPLE_BYTES {
        return None;
    }
    let body_ms = duration_ms(elapsed.saturating_sub(ttfb.unwrap_or_default()));
    Some(bytes.saturating_mul(8_000) / body_ms)
}
