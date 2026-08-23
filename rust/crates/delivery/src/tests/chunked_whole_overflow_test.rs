use super::whole_sink_fixture::{fixture, split, whole_spec, AuthorizedTraffic};
use crate::chunk::cancel::cancel_pair;
use crate::chunk::downloader::{download_chunk_captured, ChunkExecution};
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

    let observed = download_chunk_captured(
        &spec,
        ChunkExecution {
            sink: &sink,
            stats: &mut stats,
            cancel: &cancel,
            network: &network,
            traffic: &mut traffic,
            network_class: ghostr_engine::origin_model::NetworkClass::Unavailable,
        },
    )
    .await;
    assert_eq!(observed.received_bytes, 9);
    let error = observed
        .result
    .expect_err("cap+1 must fail");

    let limit = crate::chunk::whole_body_limit::from_error(&error).unwrap();
    assert_eq!(limit.maximum_bytes(), 8);
    assert_eq!(limit.received_bytes(), 9);
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
