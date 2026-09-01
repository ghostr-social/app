use super::{expect_queued, spec, ObservationFixture, ObservedTraffic};
use ghostr_delivery::chunk::downloader::{
    download_chunk_observed, ChunkExecution, ChunkResult, ChunkSink,
};
use ghostr_engine::host_stats::HostStats;
use tokio::time::Instant;

pub(super) async fn download(
    fixture: &mut ObservationFixture,
    stats: &mut HostStats,
    traffic: &mut ObservedTraffic,
) -> (ChunkResult, core::time::Duration) {
    let spec = spec(&fixture.requests, &fixture.url);
    let sink = ChunkSink {
        store: &fixture.store,
        key: "clip",
    };
    let started = Instant::now();
    let download = download_chunk_observed(
        &spec,
        ChunkExecution {
            sink: &sink,
            stats,
            cancel: &fixture.token,
            network: &fixture.network,
            traffic,
            network_class: ghostr_engine::origin_model::NetworkClass::Unavailable,
        },
    );
    tokio::pin!(download);
    expect_queued(&mut download).await;
    drop(fixture.held_other.take().expect("held queue slot"));
    let result = download
        .await
        .result
        .expect("download after local gate wait");
    (result, started.elapsed())
}
