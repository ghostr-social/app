use core::time::Duration;
use ghostr_delivery::chunk::downloader::{DownloadTraffic, OpenedResponse};

#[derive(Default)]
pub(super) struct HeaderTraffic {
    pub(super) observed: Option<OpenedResponse>,
}

impl DownloadTraffic for HeaderTraffic {
    fn opened(&mut self, _: Duration) {}
    fn wrote(&mut self, _: u64) {}
    fn response_observed(&mut self, response: OpenedResponse) {
        self.observed = Some(response);
    }
}
