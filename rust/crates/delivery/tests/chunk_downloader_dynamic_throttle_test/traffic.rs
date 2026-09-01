use ghostr_delivery::chunk::downloader::DownloadTraffic;

pub(super) struct IgnoreTraffic;

impl DownloadTraffic for IgnoreTraffic {
    fn opened(&mut self, _ttfb: core::time::Duration) {}
    fn wrote(&mut self, _bytes: u64) {}
}
