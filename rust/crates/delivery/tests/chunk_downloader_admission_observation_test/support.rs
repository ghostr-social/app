use super::super::range_fixture;
use core::time::Duration;
use ghostr_delivery::chunk::downloader::{ChunkSpec, DownloadTraffic};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::ByteRange;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::transfer_timeouts::TransferTimeouts;

pub(super) const BODY_BYTES: u64 = 32 * 1_024;

pub(super) fn executor() -> MediaRequestExecutor {
    MediaRequestExecutor::new(
        range_fixture::raw_media_client(),
        MediaRequestLimits::try_new(3, 3).expect("valid test fixture"),
    )
}

pub(super) async fn admit(
    requests: &MediaRequestExecutor,
    url: &str,
) -> ghostr_net::media_request_executor::AdmittedMediaRequest {
    requests
        .get(url, PreemptionAuthority::Transition)
        .expect("valid test fixture")
        .admit_for(Duration::from_secs(1))
        .await
        .expect("valid test fixture")
}

pub(super) async fn expect_queued<F>(future: &mut core::pin::Pin<&mut F>)
where
    F: core::future::Future,
{
    tokio::select! {
        biased;
        _ = future => panic!("download bypassed the occupied global gate"),
        () = tokio::time::sleep(Duration::from_millis(500)) => {}
    }
}

pub(super) fn spec<'a>(requests: &'a MediaRequestExecutor, url: &'a str) -> ChunkSpec<'a> {
    ChunkSpec {
        requests,
        url,
        request: range_fixture::range_request(ByteRange::new(0, BODY_BYTES)),
        continuation: None,
        priority: PreemptionAuthority::Transition,
        timeouts: TransferTimeouts::default(),
    }
}

#[derive(Default)]
pub(super) struct ObservedTraffic {
    pub(super) concurrency: usize,
}

impl DownloadTraffic for ObservedTraffic {
    fn opened(&mut self, _ttfb: Duration) {}
    fn wrote(&mut self, _bytes: u64) {}
    fn concurrency(&mut self, active: usize) {
        self.concurrency = active;
    }
}
