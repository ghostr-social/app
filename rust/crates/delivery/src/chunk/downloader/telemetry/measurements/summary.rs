use super::{HttpResponseEvidence, OpenBodyMeasurement, ResponseObservation, TrafficMeasurements};
use crate::chunk::traffic::WholeBodyCompletion;
use core::time::Duration;
use ghostr_engine::origin_model::OriginAttemptContext;

impl TrafficMeasurements {
    pub fn bytes(&self) -> u64 {
        self.bytes
    }

    pub fn origin_elapsed(&self) -> Option<Duration> {
        self.origin_elapsed
    }

    pub fn request_started(&self) -> bool {
        self.request_started
    }

    pub(in crate::chunk::downloader) const fn attempt_context(
        &self,
    ) -> Option<OriginAttemptContext> {
        self.attempt_context
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

    pub(in crate::chunk::downloader::telemetry) const fn response_observation(
        &self,
    ) -> Option<ResponseObservation> {
        self.response_observation
    }

    pub(in crate::chunk::downloader::telemetry) fn open_body(
        &self,
    ) -> Option<&OpenBodyMeasurement> {
        self.open_body.as_ref()
    }
}
