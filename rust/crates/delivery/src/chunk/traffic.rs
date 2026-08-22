use crate::chunk::downloader::{OpenedResponse, ResponseAdmission, ResponseObservation};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

pub trait ChunkTraffic: Send {
    fn current_network_class(&mut self) -> Option<ghostr_engine::origin_model::NetworkClass> {
        None
    }

    fn concurrency(&mut self, _active: usize) {}
    fn request_started(&mut self) {}
    fn opened(&mut self, ttfb: Duration);
    fn wrote(&mut self, bytes: u64);
    fn response_observed(&mut self, _response: ResponseObservation) {}

    fn authorize_response<'a>(
        &'a mut self,
        response: OpenedResponse,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<ResponseAdmission>> + Send + 'a>> {
        self.response_observed(response.observation());
        Box::pin(async { Ok(ResponseAdmission::Proceed) })
    }
}
