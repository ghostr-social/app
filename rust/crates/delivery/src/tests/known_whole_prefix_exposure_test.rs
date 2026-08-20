use super::whole_sink_fixture::{fixture, split, whole_spec, AuthorizedTraffic};
use crate::chunk::cancel::cancel_pair;
use crate::chunk::downloader::download_chunk_observed;
use crate::chunk::sink::TransferChunkSink;
use crate::debug::network::NetworkThrottle;
use ghostr_engine::adaptive::WholeBodyContract;
use ghostr_engine::host_stats::HostStats;
use std::time::Duration;

#[tokio::test]
async fn capped_whole_with_a_declared_length_exposes_its_live_prefix() {
    let prefix = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: 8\r\nETag: \"whole\"\r\n\r\nnew!";
    let origin = split(prefix, b"body").await;
    let fixture = fixture("known-whole", &origin.url, None).await;
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
    let download =
        download_chunk_observed(&spec, &sink, &mut stats, &cancel, &network, &mut traffic);
    tokio::pin!(download);

    let mut prefix_sent = origin.prefix_sent;
    tokio::select! {
        sent = &mut prefix_sent => sent.unwrap(),
        result = &mut download => panic!("download ended before prefix: {result:?}"),
    }
    let readable = tokio::time::timeout(Duration::from_secs(1), wait_for_prefix(&fixture.store));
    tokio::pin!(readable);
    tokio::select! {
        result = &mut download => panic!("download ended before release: {result:?}"),
        result = &mut readable => result.expect("declared whole prefix becomes player-readable"),
    }
    origin.release.notify_one();
    assert_eq!(download.await.unwrap().bytes_written, 8);
    fixture.store.release_action(&fixture.action).await;
    std::fs::remove_dir_all(fixture.root).unwrap();
}

async fn wait_for_prefix(store: &ghostr_partial_store::partial_range_store::PartialRangeStore) {
    let notifier = store.change_notifier();
    loop {
        let changed = notifier.notified();
        if store.read_range("post", 0..4).await.unwrap() == Some(b"new!".to_vec()) {
            return;
        }
        changed.await;
    }
}
