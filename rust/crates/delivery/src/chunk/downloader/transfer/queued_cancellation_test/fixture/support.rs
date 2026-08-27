use crate::chunk::downloader::{ChunkResult, ChunkSpec, DownloadTraffic};
use core::time::Duration;
use ghostr_engine::adaptive::{PreemptionAuthority, RetrievalRequest};
use ghostr_engine::ByteRange;
use ghostr_net::media_request_executor::MediaRequestExecutor;
use ghostr_net::transfer_timeouts::TransferTimeouts;

pub(super) const URL: &str = "https://media.example/video.mp4";

pub(super) async fn expect_pending<F>(future: &mut core::pin::Pin<&mut F>)
where
    F: core::future::Future<Output = anyhow::Result<ChunkResult>>,
{
    tokio::select! {
        biased;
        result = future => panic!("queued request completed early: {result:?}"),
        () = tokio::task::yield_now() => {}
    }
}

pub(super) fn spec(requests: &MediaRequestExecutor) -> ChunkSpec<'_> {
    ChunkSpec {
        requests,
        url: URL,
        request: RetrievalRequest::FetchRange {
            bytes: ByteRange::new(0, 1),
            promotion: None,
        },
        attempt_profile: ghostr_engine::origin_model::OriginAttemptProfile::new(
            ghostr_engine::origin_model::OriginRequestProfile::new(
                ghostr_engine::origin_model::RequestMethod::RangeGet,
                1,
                ghostr_engine::origin_model::MediaClass::ProgressiveMp4,
            ),
        ),
        priority: PreemptionAuthority::Transition,
        continuation: None,
        timeouts: TransferTimeouts::default(),
    }
}

pub(super) struct IgnoreTraffic;

impl DownloadTraffic for IgnoreTraffic {
    fn opened(&mut self, _ttfb: Duration) {}
    fn wrote(&mut self, _bytes: u64) {}
}
