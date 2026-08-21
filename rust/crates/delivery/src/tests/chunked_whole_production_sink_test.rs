use super::whole_sink_fixture::{fixture, split, whole_spec, AuthorizedTraffic};
use crate::chunk::cancel::cancel_pair;
use crate::chunk::downloader::{download_chunk_observed, ChunkExecution};
use crate::chunk::sink::TransferChunkSink;
use crate::debug::network::NetworkThrottle;
use ghostr_engine::adaptive::WholeBodyContract;
use ghostr_engine::host_stats::HostStats;

#[tokio::test]
async fn chunked_whole_keeps_seed_readable_until_atomic_eof_commit() {
    let prefix = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nTransfer-Encoding: chunked\r\nETag: \"whole\"\r\n\r\n4\r\nnew!\r\n";
    let origin = split(prefix, b"4\r\nbody\r\n0\r\n\r\n").await;
    let fixture = fixture("chunked-whole", &origin.url, Some(b"old!")).await;
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
        WholeBodyContract::Capped { maximum_bytes: 16 },
    );
    let download = download_chunk_observed(
        &spec,
        ChunkExecution {
            sink: &sink,
            stats: &mut stats,
            cancel: &cancel,
            network: &network,
            traffic: &mut traffic,
        },
    );
    tokio::pin!(download);

    let mut prefix_sent = origin.prefix_sent;
    tokio::select! {
        sent = &mut prefix_sent => sent.unwrap(),
        result = &mut download => panic!("download ended before prefix: {result:?}"),
    }
    let staged = wait_for_staging(&fixture.used);
    tokio::pin!(staged);
    tokio::select! {
        result = &mut download => panic!("download ended before release: {result:?}"),
        () = &mut staged => {}
    }
    assert_eq!(
        fixture.store.read_range("post", 0..4).await.unwrap(),
        Some(b"old!".to_vec())
    );
    origin.release.notify_one();
    assert_eq!(download.await.unwrap().total_bytes, Some(8));
    assert_eq!(
        fixture.store.read_range("post", 0..8).await.unwrap(),
        Some(b"new!body".to_vec())
    );
    fixture.store.release_action(&fixture.action).await;
    std::fs::remove_dir_all(fixture.root).unwrap();
}

async fn wait_for_staging(used: &tokio::sync::Mutex<u64>) {
    while *used.lock().await < 8 {
        tokio::task::yield_now().await;
    }
}
