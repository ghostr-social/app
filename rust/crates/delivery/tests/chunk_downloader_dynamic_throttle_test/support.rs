use core::sync::atomic::{AtomicU64, Ordering};
use ghostr_delivery::chunk::cancel::CancelToken;
use ghostr_delivery::chunk::downloader::{
    download_chunk_observed, ChunkExecution, ChunkResult, ChunkSpec, ChunkWrite, OriginGeneration,
    ResponseWriteMode,
};
use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_engine::host_stats::HostStats;
use tokio::sync::Notify;

mod traffic;
use traffic::IgnoreTraffic;

#[derive(Default)]
pub(super) struct ProgressSink {
    end: AtomicU64,
    changed: Notify,
}

pub(super) struct DownloadInput<'a> {
    pub spec: &'a ChunkSpec<'a>,
    pub sink: &'a ProgressSink,
    pub stats: &'a mut HostStats,
    pub cancel: &'a CancelToken,
    pub network: &'a NetworkThrottle,
}

impl ProgressSink {
    pub(super) fn prefix_end(&self) -> u64 {
        self.end.load(Ordering::Acquire)
    }

    pub(super) async fn wait_for_prefix(&self, minimum: u64) {
        while self.prefix_end() < minimum {
            let changed = self.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if self.prefix_end() < minimum {
                changed.await;
            }
        }
    }
}

impl ChunkWrite for ProgressSink {
    fn accept(
        &self,
        _generation: &OriginGeneration,
        _mode: ResponseWriteMode,
    ) -> impl core::future::Future<Output = anyhow::Result<()>> + Send {
        core::future::ready(Ok(()))
    }
    fn write(
        &self,
        _generation: &OriginGeneration,
        _mode: ResponseWriteMode,
        offset: u64,
        bytes: &[u8],
    ) -> impl core::future::Future<Output = anyhow::Result<bool>> + Send {
        let result = self.write_prefix(offset, bytes);
        core::future::ready(result)
    }
    fn finish(
        &self,
        _generation: &OriginGeneration,
        _mode: ResponseWriteMode,
        _total: Option<u64>,
        _complete: bool,
    ) -> impl core::future::Future<Output = anyhow::Result<bool>> + Send {
        core::future::ready(Ok(true))
    }
}
impl ProgressSink {
    fn write_prefix(&self, offset: u64, bytes: &[u8]) -> anyhow::Result<bool> {
        anyhow::ensure!(offset == self.prefix_end(), "non-contiguous test write");
        self.end
            .store(offset + bytes.len() as u64, Ordering::Release);
        self.changed.notify_waiters();
        Ok(true)
    }
}

pub(super) async fn download(input: DownloadInput<'_>) -> anyhow::Result<ChunkResult> {
    let mut traffic = IgnoreTraffic;
    download_chunk_observed(
        input.spec,
        ChunkExecution {
            sink: input.sink,
            stats: input.stats,
            cancel: input.cancel,
            network: input.network,
            traffic: &mut traffic,
            network_class: ghostr_engine::origin_model::NetworkClass::Unavailable,
        },
    )
    .await
    .result
}
