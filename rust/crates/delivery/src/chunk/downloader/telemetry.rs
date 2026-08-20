use super::{ChunkResult, ChunkSpec, OpenedResponse, ResponseAdmission, ResponseObservation};
use crate::chunk::traffic::ChunkTraffic;
use ghostr_engine::adaptive::RetrievalRequest;
use ghostr_engine::origin_model::{
    ErrorReason, MediaClass, NetworkClass, OriginContext, OriginObservation, OriginQuery,
    RequestMethod,
};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct TrafficMeasurements {
    ttfb: Option<Duration>,
    bytes: u64,
}

pub(super) struct MeasuredTraffic<'a> {
    inner: &'a mut dyn ChunkTraffic,
    measured: TrafficMeasurements,
}

impl<'a> MeasuredTraffic<'a> {
    pub fn new(inner: &'a mut dyn ChunkTraffic) -> Self {
        Self {
            inner,
            measured: TrafficMeasurements::default(),
        }
    }

    pub fn measurements(&self) -> TrafficMeasurements {
        self.measured
    }
}

impl ChunkTraffic for MeasuredTraffic<'_> {
    fn opened(&mut self, ttfb: Duration) {
        self.measured.ttfb = Some(ttfb);
        self.inner.opened(ttfb);
    }

    fn wrote(&mut self, bytes: u64) {
        self.measured.bytes = self.measured.bytes.saturating_add(bytes);
        self.inner.wrote(bytes);
    }

    fn response_observed(&mut self, response: ResponseObservation) {
        self.inner.response_observed(response);
    }

    fn authorize_response<'a>(
        &'a mut self,
        response: OpenedResponse,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<ResponseAdmission>> + Send + 'a>> {
        self.inner.authorize_response(response)
    }
}

pub(super) fn observation(
    spec: &ChunkSpec<'_>,
    result: &anyhow::Result<ChunkResult>,
    measured: TrafficMeasurements,
    timing: ObservationTiming,
) -> OriginObservation {
    let query = OriginQuery::new(spec.url, context(spec, timing));
    let mut item = match result {
        Ok(result) if result.cancelled => OriginObservation::cancelled(query, timing.at_ms),
        Ok(_) => OriginObservation::success(query, timing.at_ms),
        Err(error) => OriginObservation::failure(query, timing.at_ms, error_reason(error)),
    };
    item.range_compliant = range_compliance(spec.request, result);
    item.ttfb_ms = measured.ttfb.map(duration_ms);
    item.throughput_bps = throughput(measured.bytes, timing.elapsed, measured.ttfb);
    item
}

#[derive(Clone, Copy)]
pub(super) struct ObservationTiming {
    pub at_ms: u64,
    pub elapsed: Duration,
    pub concurrency: usize,
}

fn context(spec: &ChunkSpec<'_>, timing: ObservationTiming) -> OriginContext {
    OriginContext::new(
        method(spec.request),
        spec.request.requested_bytes().len(),
        media_class(spec.request),
    )
    .with_network(NetworkClass::Unavailable)
    .with_concurrency(timing.concurrency)
    .with_observed_at_ms(timing.at_ms)
}

fn method(request: RetrievalRequest) -> RequestMethod {
    match request {
        RetrievalRequest::FetchRange { .. } => RequestMethod::RangeGet,
        RetrievalRequest::FetchWhole { .. } => RequestMethod::FullGet,
    }
}

fn media_class(request: RetrievalRequest) -> MediaClass {
    match request {
        RetrievalRequest::FetchRange { .. } => MediaClass::ProgressiveMp4,
        RetrievalRequest::FetchWhole { .. } => MediaClass::WholeObject,
    }
}

fn range_compliance(
    request: RetrievalRequest,
    result: &anyhow::Result<ChunkResult>,
) -> Option<bool> {
    if !matches!(request, RetrievalRequest::FetchRange { .. }) {
        return None;
    }
    match result {
        Ok(result) => result
            .range_support
            .or(result.range_ignored.then_some(false)),
        Err(error) if error_reason(error) == ErrorReason::RangeNoncompliant => Some(false),
        Err(_) => None,
    }
}

fn throughput(bytes: u64, elapsed: Duration, ttfb: Option<Duration>) -> Option<u64> {
    if bytes == 0 {
        return None;
    }
    let body = elapsed.saturating_sub(ttfb.unwrap_or_default());
    let seconds = body.max(Duration::from_millis(1)).as_secs_f64();
    Some((bytes as f64 / seconds).round().clamp(1.0, u64::MAX as f64) as u64)
}

fn duration_ms(value: Duration) -> u64 {
    value.as_millis().min(u128::from(u64::MAX)).max(1) as u64
}

fn error_reason(error: &anyhow::Error) -> ErrorReason {
    if let Some(status) = error.chain().find_map(reqwest_status) {
        return match status.is_server_error() {
            true => ErrorReason::Http5xx,
            false => ErrorReason::Http4xx,
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

fn reqwest_status(error: &(dyn std::error::Error + 'static)) -> Option<reqwest::StatusCode> {
    error
        .downcast_ref::<reqwest::Error>()
        .and_then(reqwest::Error::status)
}
