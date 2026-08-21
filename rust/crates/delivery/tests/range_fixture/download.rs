use ghostr_delivery::chunk::cancel::CancelToken;
use ghostr_delivery::chunk::downloader::{
    download_chunk_observed, ChunkExecution, ChunkResult, ChunkSink, ChunkSpec, DownloadTraffic,
};
use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_engine::host_stats::HostStats;
use std::time::Duration;

pub struct DownloadContext<'a> {
    stats: &'a mut HostStats,
    cancel: &'a CancelToken,
    network: &'a NetworkThrottle,
}

pub fn context<'a>(
    stats: &'a mut HostStats,
    cancel: &'a CancelToken,
    network: &'a NetworkThrottle,
) -> DownloadContext<'a> {
    DownloadContext {
        stats,
        cancel,
        network,
    }
}

pub async fn download_chunk_throttled(
    spec: &ChunkSpec<'_>,
    sink: &ChunkSink<'_>,
    context: DownloadContext<'_>,
) -> anyhow::Result<ChunkResult> {
    download_chunk_observed(
        spec,
        ChunkExecution {
            sink,
            stats: context.stats,
            cancel: context.cancel,
            network: context.network,
            traffic: &mut IgnoreTraffic,
        },
    )
    .await
}

struct IgnoreTraffic;

impl DownloadTraffic for IgnoreTraffic {
    fn opened(&mut self, _ttfb: Duration) {}

    fn wrote(&mut self, _bytes: u64) {}
}
