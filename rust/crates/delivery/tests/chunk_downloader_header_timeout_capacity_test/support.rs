use super::range_fixture;
use core::time::Duration;
use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{ChunkResult, ChunkSink, ChunkSpec};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::ByteRange;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaResponse};
use ghostr_net::transfer_timeouts::TransferTimeouts;
use std::path::PathBuf;

const SHORT_DEADLINE: Duration = Duration::from_millis(60);

pub(super) struct DownloadFixture<'a> {
    requests: &'a MediaRequestExecutor,
    store: ghostr_partial_store::partial_range_store::PartialRangeStore,
    network: ghostr_delivery::debug::network::NetworkThrottle,
    _root: TempRoot,
}

impl<'a> DownloadFixture<'a> {
    pub(super) fn new(requests: &'a MediaRequestExecutor) -> Self {
        let root = TempRoot(range_fixture::temp_root("ghostr-header-timeout"));
        Self {
            requests,
            store: range_fixture::store(root.0.clone()),
            network: range_fixture::network(),
            _root: root,
        }
    }

    pub(super) async fn download(&self, url: &str) -> anyhow::Result<ChunkResult> {
        let (_handle, token) = cancel_pair();
        let mut stats = HostStats::new();
        let spec = self.spec(url);
        let sink = ChunkSink {
            store: &self.store,
            key: "stalled",
        };
        range_fixture::download_chunk_throttled(
            &spec,
            &sink,
            range_fixture::context(&mut stats, &token, &self.network),
        )
        .await
    }

    fn spec(&self, url: &'a str) -> ChunkSpec<'a> {
        ChunkSpec {
            requests: self.requests,
            url,
            request: range_fixture::range_request(ByteRange::new(0, 8)),
            attempt_profile: range_fixture::range_profile(8),
            continuation: None,
            priority: PreemptionAuthority::Transition,
            timeouts: TransferTimeouts {
                admission: SHORT_DEADLINE,
                headers: SHORT_DEADLINE,
                idle: SHORT_DEADLINE,
            },
        }
    }
}

pub(super) async fn open_healthy(
    requests: &MediaRequestExecutor,
    url: &str,
) -> anyhow::Result<MediaResponse> {
    requests
        .get(url, PreemptionAuthority::Transition)?
        .admit_for(Duration::from_millis(250))
        .await?
        .send_with_redirect_deadline(tokio::time::Instant::now() + Duration::from_secs(1))
        .await
}

struct TempRoot(PathBuf);

impl Drop for TempRoot {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).ok();
    }
}
