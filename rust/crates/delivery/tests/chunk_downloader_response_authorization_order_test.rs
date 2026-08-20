mod range_fixture;

use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{
    download_chunk_observed, ChunkSpec, ChunkWrite, DownloadTraffic, OriginGeneration,
    ResponseObservation, ResponseWriteMode,
};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::ByteRange;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn response_semantics_are_authorized_before_store_admission() {
    let origin = range_fixture::ranged::serve_ranged(b"abcdefgh".to_vec()).await;
    let client = range_fixture::media_client();
    let accepted = Arc::new(AtomicBool::new(false));
    let observed_first = Arc::new(AtomicBool::new(false));
    let sink = OrderingSink(accepted.clone());
    let mut traffic = OrderingTraffic(accepted, observed_first.clone());
    let (_, token) = cancel_pair();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        client: &client,
        url: &origin,
        request: range_fixture::range_request(ByteRange::new(0, 8)),
        continuation: None,
        timeouts: Default::default(),
    };

    download_chunk_observed(
        &spec,
        &sink,
        &mut stats,
        &token,
        &range_fixture::network(),
        &mut traffic,
    )
    .await
    .unwrap();

    assert!(observed_first.load(Ordering::Acquire));
}

struct OrderingSink(Arc<AtomicBool>);

impl ChunkWrite for OrderingSink {
    async fn accept<'a>(
        &'a self,
        _: &'a OriginGeneration,
        _: ResponseWriteMode,
    ) -> anyhow::Result<()> {
        self.0.store(true, Ordering::Release);
        Ok(())
    }

    async fn write<'a>(
        &'a self,
        _: &'a OriginGeneration,
        _: ResponseWriteMode,
        _: u64,
        _: &'a [u8],
    ) -> anyhow::Result<bool> {
        Ok(true)
    }

    async fn finish<'a>(
        &'a self,
        _: &'a OriginGeneration,
        _: ResponseWriteMode,
        _: Option<u64>,
        _: bool,
    ) -> anyhow::Result<bool> {
        Ok(true)
    }
}

struct OrderingTraffic(Arc<AtomicBool>, Arc<AtomicBool>);

impl DownloadTraffic for OrderingTraffic {
    fn opened(&mut self, _: Duration) {}
    fn wrote(&mut self, _: u64) {}

    fn response_observed(&mut self, _: ResponseObservation) {
        self.1
            .store(!self.0.load(Ordering::Acquire), Ordering::Release);
    }
}
