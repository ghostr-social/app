use ghostr_delivery::chunk::cancel::CancelToken;
use ghostr_delivery::chunk::downloader::{
    download_chunk_observed, ChunkExecution, ChunkResult, ChunkSink, ChunkSpec, DownloadTraffic,
};
use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_engine::host_stats::HostStats;

mod traffic;
use traffic::IgnoreTraffic;
pub use traffic::ObservationTraffic;

pub fn rejection_traffic() -> impl DownloadTraffic {
    traffic::RejectTraffic
}

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
    download_chunk_with_traffic(spec, sink, context, &mut IgnoreTraffic).await
}

pub async fn download_chunk_with_traffic(
    spec: &ChunkSpec<'_>,
    sink: &ChunkSink<'_>,
    context: DownloadContext<'_>,
    traffic: &mut dyn DownloadTraffic,
) -> anyhow::Result<ChunkResult> {
    download_chunk_observed(
        spec,
        ChunkExecution {
            sink,
            stats: context.stats,
            cancel: context.cancel,
            network: context.network,
            traffic,
            network_class: ghostr_engine::origin_model::NetworkClass::Unavailable,
        },
    )
    .await
    .result
}
