use super::{expect_queued, spec, ObservationFixture, ObservedTraffic};
use ghostr_delivery::chunk::downloader::{download_chunk_observed, ChunkExecution, ChunkSink};
use ghostr_engine::host_stats::HostStats;

pub(super) async fn download(
    fixture: &mut ObservationFixture,
    stats: &mut HostStats,
    traffic: &mut ObservedTraffic,
) {
    let spec = spec(&fixture.requests, &fixture.url);
    let sink = ChunkSink {
        store: &fixture.store,
        key: "clip",
    };
    let download = download_chunk_observed(
        &spec,
        ChunkExecution {
            sink: &sink,
            stats,
            cancel: &fixture.token,
            network: &fixture.network,
            traffic,
        },
    );
    tokio::pin!(download);
    expect_queued(&mut download).await;
    drop(fixture.held_other.take().expect("held queue slot"));
    download.await.expect("download after local gate wait");
}
