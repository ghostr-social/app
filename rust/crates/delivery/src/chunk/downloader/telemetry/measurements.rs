use super::super::{OpenedResponse, ResponseAdmission, ResponseObservation};
use crate::chunk::traffic::ChunkTraffic;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tokio::time::Instant;

#[derive(Clone, Copy, Debug)]
pub(in crate::chunk::downloader) struct TrafficMeasurements {
    pub(super) ttfb: Option<Duration>,
    pub(super) bytes: u64,
    concurrency: usize,
    request_started: bool,
    origin_elapsed: Option<Duration>,
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
        }
    }
}

impl<'a> MeasuredTraffic<'a> {
    pub fn new(inner: &'a mut dyn ChunkTraffic) -> Self {
        Self {
            inner,
            measured: TrafficMeasurements::default(),
            opened_at: None,
        }
    }

    pub fn measurements(&self) -> TrafficMeasurements {
        let mut measured = self.measured;
        measured.origin_elapsed = self
            .opened_at
            .zip(measured.ttfb)
            .map(|(opened_at, ttfb)| ttfb + opened_at.elapsed());
        measured
    }
}

impl TrafficMeasurements {
    pub fn concurrency(self) -> usize {
        self.concurrency
    }

    pub fn origin_elapsed(self) -> Option<Duration> {
        self.origin_elapsed
    }

    pub fn request_started(self) -> bool {
        self.request_started
    }
}

impl ChunkTraffic for MeasuredTraffic<'_> {
    fn concurrency(&mut self, active: usize) {
        self.measured.concurrency = active.max(1);
        self.inner.concurrency(active);
    }

    fn request_started(&mut self) {
        self.measured.request_started = true;
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
