mod delivery_fixture;
mod raw_http;

use delivery_fixture::{media_client, temp_directory};
use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{download_chunk_throttled, ChunkSink, ChunkSpec};
use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use raw_http::spawn_raw_server;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn chunk_downloader_rejects_partial_content_without_a_content_range() {
    let response =
        b"HTTP/1.1 206 Partial Content\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo";
    let (url, request) = spawn_raw_server(response).await;
    let root = temp_directory("ghostr-missing-content-range");
    let store = PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    );
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        client: &media_client(),
        url: &url,
        range: ByteRange::new(0, 5),
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };

    let error = download_chunk_throttled(&spec, &sink, &mut stats, &token, &NetworkThrottle::new())
        .await
        .expect_err("malformed partial response must fail");

    assert!(error.to_string().contains("missing Content-Range"));
    assert!(store
        .present_ranges("clip")
        .await
        .expect("ranges")
        .is_empty());
    request.await.expect("upstream request");
    std::fs::remove_dir_all(root).ok();
}
