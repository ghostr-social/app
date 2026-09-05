use super::{ChunkResult, ChunkSpec};
use core::time::Duration;
use ghostr_engine::adaptive::RetrievalRequest;
use ghostr_engine::origin_model::{
    ErrorReason, OpenBodyObservation, OriginContext, OriginObservation, OriginOutcome, OriginQuery,
};

mod measurements;
#[cfg(test)]
mod rejection_reason_test;
#[cfg(test)]
#[path = "telemetry/request_start_context_test.rs"]
mod request_start_context_test;
#[cfg(test)]
#[path = "telemetry/response_semantics_test.rs"]
mod response_semantics_test;
#[cfg(test)]
#[path = "telemetry/throughput_unit_test.rs"]
mod throughput_unit_test;

pub(super) use measurements::{MeasuredTraffic, TrafficMeasurements};

pub(super) fn observation(
    spec: &ChunkSpec<'_>,
    result: &anyhow::Result<ChunkResult>,
    measured: &TrafficMeasurements,
    timing: ObservationTiming,
) -> OriginObservation {
    let query = OriginQuery::new(spec.url, context(measured, timing));
    let mut item = match result {
        Ok(result) if result.cancelled => OriginObservation::cancelled(query, timing.at_ms),
        Ok(_) => OriginObservation::success(query, timing.at_ms),
        Err(error) if crate::chunk::sink::is_local_store_failure(error) => {
            OriginObservation::success(query, timing.at_ms)
        }
        Err(error) if crate::chunk::whole_body_policy::is(error) => {
            OriginObservation::success(query, timing.at_ms)
        }
        Err(error) => OriginObservation::failure(query, timing.at_ms, error_reason(error)),
    };
    item.range_compliant = range_compliance(spec.request, result, measured.response_observation());
    item.ttfb_ms = measured.ttfb.map(duration_ms);
    item.throughput_bps = throughput(measured.bytes, timing.elapsed, measured.ttfb);
    item
}

pub(super) fn open_body_observation(
    spec: &ChunkSpec<'_>,
    result: &anyhow::Result<ChunkResult>,
    measured: &TrafficMeasurements,
    observed_at_ms: u64,
) -> Option<OpenBodyObservation> {
    let body = measured.open_body()?;
    let context = measured
        .attempt_context()?
        .request_context()
        .with_planned_bytes(body.planned_bytes());
    let query = OriginQuery::new(spec.url, context);
    let mut item = body_item(query, result, measured, observed_at_ms);
    item.throughput_bps = throughput(body.received_bytes(measured.bytes), body.elapsed(), None);
    Some(item)
}

fn body_item(
    query: OriginQuery,
    result: &anyhow::Result<ChunkResult>,
    measured: &TrafficMeasurements,
    observed_at_ms: u64,
) -> OpenBodyObservation {
    match body_outcome(result, measured) {
        OriginOutcome::Success => OpenBodyObservation::success(query, observed_at_ms),
        OriginOutcome::Failure(reason) => {
            OpenBodyObservation::failure(query, observed_at_ms, reason)
        }
        OriginOutcome::Cancelled => OpenBodyObservation::cancelled(query, observed_at_ms),
    }
}

fn body_outcome(
    result: &anyhow::Result<ChunkResult>,
    measured: &TrafficMeasurements,
) -> OriginOutcome {
    if measured.whole_body_completion().is_some() {
        return OriginOutcome::Success;
    }
    match result {
        Ok(item) if item.cancelled => OriginOutcome::Cancelled,
        Ok(_) => OriginOutcome::Failure(ErrorReason::Unknown),
        Err(error) if censored_body_error(error) => OriginOutcome::Cancelled,
        Err(error) => OriginOutcome::Failure(error_reason(error)),
    }
}

fn censored_body_error(error: &anyhow::Error) -> bool {
    crate::chunk::sink::is_local_store_failure(error) || crate::chunk::whole_body_policy::is(error)
}

#[derive(Clone, Copy)]
pub(super) struct ObservationTiming {
    pub at_ms: u64,
    pub elapsed: Duration,
}

fn context(measured: &TrafficMeasurements, _timing: ObservationTiming) -> OriginContext {
    measured
        .attempt_context()
        .expect("origin observations require a started request")
        .request_context()
}

fn range_compliance(
    request: RetrievalRequest,
    result: &anyhow::Result<ChunkResult>,
    observed: Option<super::ResponseObservation>,
) -> Option<bool> {
    if !matches!(request, RetrievalRequest::FetchRange { .. }) {
        return None;
    }
    if result
        .as_ref()
        .err()
        .is_some_and(|error| error_reason(error) == ErrorReason::RangeNoncompliant)
    {
        return Some(false);
    }
    match observed {
        Some(super::ResponseObservation::Partial { .. }) => return Some(true),
        Some(
            super::ResponseObservation::Body { .. } | super::ResponseObservation::Ignored { .. },
        ) => return Some(false),
        Some(super::ResponseObservation::Rejected(_)) | None => {}
    }
    match result {
        Ok(result) => result
            .range_support
            .or_else(|| result.range_ignored.then_some(false)),
        Err(_) => None,
    }
}

fn throughput(bytes: u64, elapsed: Duration, ttfb: Option<Duration>) -> Option<u64> {
    if bytes == 0 {
        return None;
    }
    let body = elapsed.saturating_sub(ttfb.unwrap_or_default());
    let millis = body.as_millis().max(1).min(u128::from(u64::MAX)) as u64;
    Some(bytes.saturating_mul(8_000) / millis)
}

fn duration_ms(value: Duration) -> u64 {
    value.as_millis().min(u128::from(u64::MAX)).max(1) as u64
}

fn error_reason(error: &anyhow::Error) -> ErrorReason {
    if let Some(reason) = error.downcast_ref::<super::ResponseFailure>() {
        return match reason {
            super::ResponseFailure::RangeNoncompliant => ErrorReason::RangeNoncompliant,
            super::ResponseFailure::InvalidResponse => ErrorReason::InvalidResponse,
        };
    }
    if let Some(status) = error.chain().find_map(reqwest_status) {
        return if status.is_server_error() {
            ErrorReason::Http5xx
        } else {
            ErrorReason::Http4xx
        };
    }
    let text = format!("{error:#}").to_ascii_lowercase();
    if text.contains("timed out") || text.contains("timeout") {
        return ErrorReason::Timeout;
    }
    if text.contains("dns") || text.contains("lookup address") {
        return ErrorReason::Dns;
    }
    if text.contains("certificate") || text.contains("tls") {
        return ErrorReason::Tls;
    }
    if text.contains("range") || text.contains("content-range") {
        return ErrorReason::RangeNoncompliant;
    }
    if text.contains("connection") || text.contains("reset by peer") {
        return ErrorReason::Connection;
    }
    ErrorReason::Unknown
}

fn reqwest_status(error: &(dyn core::error::Error + 'static)) -> Option<reqwest::StatusCode> {
    error
        .downcast_ref::<reqwest::Error>()
        .and_then(reqwest::Error::status)
}
