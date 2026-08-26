use super::super::{HttpResponseEvidence, OpenedResponse, ResponseAdmission};
use crate::chunk::traffic::{ChunkTraffic, WholeBodyCompletion};
use core::future::Future;
use core::pin::Pin;
use core::time::Duration;
use tokio::time::Instant;

#[derive(Clone, Debug)]
pub(in crate::chunk::downloader) struct TrafficMeasurements {
    pub(super) ttfb: Option<Duration>,
    pub(super) bytes: u64,
    concurrency: usize,
    request_started: bool,
    origin_elapsed: Option<Duration>,
    network_class: ghostr_engine::origin_model::NetworkClass,
    whole_body_completion: Option<WholeBodyCompletion>,
    response_evidence: Option<HttpResponseEvidence>,
}

pub(in crate::chunk::downloader) struct MeasuredTraffic<'a> {
    inner: &'a mut dyn ChunkTraffic,
    measured: TrafficMeasurements,
    opened_at: Option<Instant>,
}

impl Default for TrafficMeasurements {
    fn default() -> Self {
        Self {
            ttfb: None,
            bytes: 0,
            concurrency: 1,
            request_started: false,
            origin_elapsed: None,
            network_class: ghostr_engine::origin_model::NetworkClass::Unavailable,
            whole_body_completion: None,
            response_evidence: None,
        }
    }
}

impl<'a> MeasuredTraffic<'a> {
    pub fn new(
        inner: &'a mut dyn ChunkTraffic,
        network_class: ghostr_engine::origin_model::NetworkClass,
    ) -> Self {
        let measured = TrafficMeasurements {
            network_class,
            ..TrafficMeasurements::default()
        };
        Self {
            inner,
            measured,
            opened_at: None,
        }
    }

    pub fn measurements(&self) -> TrafficMeasurements {
        let mut measured = self.measured.clone();
        measured.origin_elapsed = measured.origin_elapsed.or_else(|| {
            self.opened_at
                .zip(measured.ttfb)
                .map(|(opened_at, ttfb)| ttfb + opened_at.elapsed())
        });
        measured
    }
}

impl TrafficMeasurements {
    pub fn concurrency(&self) -> usize {
        self.concurrency
    }

    pub fn bytes(&self) -> u64 {
        self.bytes
    }

    pub fn origin_elapsed(&self) -> Option<Duration> {
        self.origin_elapsed
    }

    pub fn request_started(&self) -> bool {
        self.request_started
    }

    pub fn network_class(&self) -> ghostr_engine::origin_model::NetworkClass {
        self.network_class
    }

    pub fn with_network_class(
        mut self,
        network_class: ghostr_engine::origin_model::NetworkClass,
    ) -> Self {
        self.network_class = network_class;
        self
    }

    pub fn whole_body_completion(&self) -> Option<&WholeBodyCompletion> {
        self.whole_body_completion.as_ref()
    }

    pub fn response_evidence(&self) -> Option<&HttpResponseEvidence> {
        self.response_evidence.as_ref()
    }
}

impl ChunkTraffic for MeasuredTraffic<'_> {
    fn concurrency(&mut self, active: usize) {
        self.measured.concurrency = active.max(1);
        self.inner.concurrency(active);
    }

    fn request_started(&mut self) {
        self.measured.request_started = true;
        if let Some(network_class) = self.inner.current_network_class() {
            self.measured.network_class = network_class;
        }
        self.inner.request_started();
    }

    fn opened(&mut self, ttfb: Duration) {
        self.measured.ttfb = Some(ttfb);
        self.opened_at = Some(Instant::now());
        self.inner.opened(ttfb);
    }

    fn wrote(&mut self, bytes: u64) {
        self.measured.bytes = self.measured.bytes.saturating_add(bytes);
        self.inner.wrote(bytes);
    }

    fn response_observed(&mut self, response: OpenedResponse) {
        self.measured.response_evidence = Some(response.evidence().clone());
        self.inner.response_observed(response);
    }

    fn whole_body_completed(&mut self, completion: WholeBodyCompletion) {
        self.measured.origin_elapsed = self
            .opened_at
            .zip(self.measured.ttfb)
            .map(|(opened_at, ttfb)| ttfb + opened_at.elapsed());
        self.measured.whole_body_completion = Some(completion.clone());
        self.inner.whole_body_completed(completion);
    }

    fn authorize_response<'a>(
        &'a mut self,
        response: OpenedResponse,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<ResponseAdmission>> + Send + 'a>> {
        self.inner.authorize_response(response)
    }
}
