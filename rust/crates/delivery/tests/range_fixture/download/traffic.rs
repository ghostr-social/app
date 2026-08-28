use core::future::Future;
use core::pin::Pin;
use core::time::Duration;
use ghostr_delivery::chunk::downloader::{
    DownloadTraffic, OpenedResponse, ResponseAdmission, ResponseObservation,
};

#[derive(Default)]
pub struct ObservationTraffic {
    observation: Option<ResponseObservation>,
}

impl ObservationTraffic {
    pub const fn observation(&self) -> Option<ResponseObservation> {
        self.observation
    }
}

impl DownloadTraffic for ObservationTraffic {
    fn opened(&mut self, _ttfb: Duration) {}
    fn wrote(&mut self, _bytes: u64) {}

    fn response_observed(&mut self, response: OpenedResponse) {
        self.observation = Some(response.observation());
    }
}

pub(super) struct RejectTraffic;

impl DownloadTraffic for RejectTraffic {
    fn opened(&mut self, _ttfb: Duration) {}
    fn wrote(&mut self, _bytes: u64) {}

    fn authorize_response<'a>(
        &'a mut self,
        _response: OpenedResponse,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<ResponseAdmission>> + Send + 'a>> {
        Box::pin(async { Ok(ResponseAdmission::Reject) })
    }
}

pub(super) struct IgnoreTraffic;

impl DownloadTraffic for IgnoreTraffic {
    fn opened(&mut self, _ttfb: Duration) {}
    fn wrote(&mut self, _bytes: u64) {}
}
