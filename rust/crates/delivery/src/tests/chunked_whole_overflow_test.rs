use super::whole_sink_fixture::{fixture, split, whole_spec, AuthorizedTraffic};
use crate::chunk::cancel::cancel_pair;
use crate::chunk::downloader::{download_chunk_observed, ChunkExecution};
use crate::chunk::sink::TransferChunkSink;
use crate::debug::network::NetworkThrottle;
use ghostr_engine::adaptive::WholeBodyContract;
use ghostr_engine::host_stats::HostStats;

#[tokio::test]
async fn chunked_whole_above_its_cap_rolls_back_without_harming_seed() {
    let response = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nTransfer-Encoding: chunked\r\n\r\n9\r\nnew body!\r\n0\r\n\r\n";
    let origin = split(response, b"").await;
    origin.release.notify_one();
    let fixture = fixture("whole-overflow", &origin.url, Some(b"old!")).await;
    let sink = TransferChunkSink::new(
        &fixture.store,
        fixture.identity.clone(),
        fixture.action.clone(),
    );
    let (_handle, cancel) = cancel_pair();
    let mut stats = HostStats::new();
    let mut traffic = AuthorizedTraffic::new(
        fixture.store.clone(),
        fixture.identity.clone(),
        fixture.action.clone(),
    );
    let network = NetworkThrottle::new();
    let spec = whole_spec(
        &fixture.client,
        &origin.url,
        WholeBodyContract::Capped { maximum_bytes: 8 },
    );

    let error = download_chunk_observed(
        &spec,
        ChunkExecution {
            sink: &sink,
            stats: &mut stats,
            cancel: &cancel,
            network: &network,
            traffic: &mut traffic,
        },
    )
    .await
    .expect_err("cap+1 must fail");

    assert!(format!("{error:#}").contains("hard cap"));
    assert_eq!(
        fixture.store.read_range("post", 0..4).await.unwrap(),
        Some(b"old!".to_vec())
    );
    assert_eq!(*fixture.used.lock().await, 4);
    fixture.store.release_action(&fixture.action).await;
    assert!(fixture
        .store
        .begin_single_response(
            &fixture.identity,
            2,
            WholeBodyContract::Capped { maximum_bytes: 8 },
        )
        .await
        .unwrap());
    std::fs::remove_dir_all(fixture.root).unwrap();
}
