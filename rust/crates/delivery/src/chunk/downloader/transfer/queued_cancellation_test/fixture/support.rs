use crate::chunk::downloader::{ChunkResult, ChunkSpec, DownloadTraffic};
use ghostr_engine::adaptive::{PreemptionAuthority, RetrievalRequest};
use ghostr_engine::ByteRange;
use ghostr_net::media_request_executor::MediaRequestExecutor;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use std::time::Duration;

pub(super) const URL: &str = "https://media.example/video.mp4";

pub(super) async fn expect_pending<F>(future: &mut std::pin::Pin<&mut F>)
where
    F: std::future::Future<Output = anyhow::Result<ChunkResult>>,
{
    tokio::select! {
        biased;
        result = future => panic!("queued request completed early: {result:?}"),
        _ = tokio::task::yield_now() => {}
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
