mod range_fixture;

use core::sync::atomic::{AtomicBool, Ordering};
use core::time::Duration;
use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{
    download_chunk_observed, ChunkExecution, ChunkSpec, ChunkWrite, DownloadTraffic,
    OpenedResponse, OriginGeneration, ResponseWriteMode,
};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::ByteRange;
use std::sync::Arc;

#[tokio::test]
async fn response_semantics_are_authorized_before_store_admission() {
    let origin = range_fixture::ranged::serve_ranged(b"abcdefgh".to_vec()).await;
    let client = range_fixture::media_client();
    let accepted = Arc::new(AtomicBool::new(false));
    let observed_first = Arc::new(AtomicBool::new(false));
    let sink = OrderingSink(std::sync::Arc::clone(&accepted));
    let mut traffic = OrderingTraffic(accepted, std::sync::Arc::clone(&observed_first));
    let (_, token) = cancel_pair();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        requests: &client,
        url: &origin,
        request: range_fixture::range_request(ByteRange::new(0, 8)),
        attempt_profile: range_fixture::range_profile(8),
        continuation: None,
        priority: ghostr_engine::adaptive::PreemptionAuthority::Transition,
        timeouts: Default::default(),
    };

    let network = range_fixture::network();
    download_chunk_observed(
        &spec,
        ChunkExecution {
            sink: &sink,
            stats: &mut stats,
            cancel: &token,
            network: &network,
            traffic: &mut traffic,
            network_class: ghostr_engine::origin_model::NetworkClass::Unavailable,
        },
    )
    .await
    .result
    .expect("valid test fixture");

    assert!(observed_first.load(Ordering::Acquire));
}

struct OrderingSink(Arc<AtomicBool>);

impl ChunkWrite for OrderingSink {
    fn accept<'a>(
        &'a self,
        _: &'a OriginGeneration,
        _: ResponseWriteMode,
    ) -> impl core::future::Future<Output = anyhow::Result<()>> + Send + 'a {
        self.0.store(true, Ordering::Release);
        core::future::ready(Ok(()))
    }

    fn write<'a>(
        &'a self,
        _: &'a OriginGeneration,
        _: ResponseWriteMode,
        _: u64,
        _: &'a [u8],
    ) -> impl core::future::Future<Output = anyhow::Result<bool>> + Send + 'a {
        core::future::ready(Ok(true))
    }

    fn finish<'a>(
        &'a self,
        _: &'a OriginGeneration,
        _: ResponseWriteMode,
        _: Option<u64>,
        _: bool,
    ) -> impl core::future::Future<Output = anyhow::Result<bool>> + Send + 'a {
        core::future::ready(Ok(true))
    }
}

struct OrderingTraffic(Arc<AtomicBool>, Arc<AtomicBool>);

impl DownloadTraffic for OrderingTraffic {
    fn opened(&mut self, _: Duration) {}
    fn wrote(&mut self, _: u64) {}

    fn response_observed(&mut self, _: OpenedResponse) {
        self.1
            .store(!self.0.load(Ordering::Acquire), Ordering::Release);
    }
}
