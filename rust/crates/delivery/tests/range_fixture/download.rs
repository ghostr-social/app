use core::time::Duration;
use ghostr_delivery::chunk::cancel::CancelToken;
use ghostr_delivery::chunk::downloader::{
    download_chunk_observed, ChunkExecution, ChunkResult, ChunkSink, ChunkSpec, DownloadTraffic,
    OpenedResponse, ResponseObservation,
};
use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_engine::host_stats::HostStats;

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

struct IgnoreTraffic;

impl DownloadTraffic for IgnoreTraffic {
    fn opened(&mut self, _ttfb: Duration) {}

    fn wrote(&mut self, _bytes: u64) {}
}
