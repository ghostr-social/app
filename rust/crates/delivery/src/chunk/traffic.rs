use crate::chunk::downloader::{HttpResponseEvidence, OpenedResponse, ResponseAdmission};
use ghostr_engine::catalog::CompleteBytesObservation;
use std::future::Future;
use std::num::NonZeroU64;
use std::pin::Pin;
use std::time::Duration;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WholeBodyCompletion {
    observation: CompleteBytesObservation,
}

impl WholeBodyCompletion {
    pub(crate) fn at_network_eof(total_bytes: NonZeroU64, response: &HttpResponseEvidence) -> Self {
        Self {
            observation: CompleteBytesObservation::new(
                total_bytes,
                response.final_url.clone(),
                crate::manager::time::evidence_time(),
                response.validator.clone(),
            ),
        }
    }

    pub(crate) const fn total_bytes(&self) -> u64 {
        self.observation.total_bytes.get()
    }

    pub(crate) fn observation(&self) -> &CompleteBytesObservation {
        &self.observation
    }
}

pub trait ChunkTraffic: Send {
    fn current_network_class(&mut self) -> Option<ghostr_engine::origin_model::NetworkClass> {
        None
    }

    fn concurrency(&mut self, _active: usize) {}
    fn request_started(&mut self) {}
    fn opened(&mut self, ttfb: Duration);
    fn wrote(&mut self, bytes: u64);
    fn received(&mut self, bytes: u64) {
        self.wrote(bytes);
    }
    fn response_observed(&mut self, _response: OpenedResponse) {}
    fn whole_body_completed(&mut self, _completion: WholeBodyCompletion) {}

    fn authorize_response<'a>(
        &'a mut self,
        _response: OpenedResponse,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<ResponseAdmission>> + Send + 'a>> {
        Box::pin(async { Ok(ResponseAdmission::Proceed) })
    }
}
