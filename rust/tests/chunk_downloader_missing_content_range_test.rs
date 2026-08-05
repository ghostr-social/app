mod support;

use rust_lib_ghostr::engine::host_stats::HostStats;
use rust_lib_ghostr::engine::ByteRange;
use rust_lib_ghostr::video::chunk_cancel::cancel_pair;
use rust_lib_ghostr::video::chunk_downloader::{download_chunk, ChunkSink, ChunkSpec};
use rust_lib_ghostr::video::partial_range_store::PartialRangeStore;
use rust_lib_ghostr::video::transfer_timeouts::TransferTimeouts;
use std::sync::Arc;
use support::fixtures::{temp_directory, trusted_media_client};
use support::http::spawn_raw_server;
use tokio::sync::Mutex;

#[tokio::test]
async fn chunk_downloader_rejects_partial_content_without_a_content_range() {
    let response =
        b"HTTP/1.1 206 Partial Content\r\nContent-Length: 5\r\nConnection: close\r\n\r\nvideo";
    let (url, request) = spawn_raw_server(response).await;
    let root = temp_directory("ghostr-missing-content-range");
    let store = PartialRangeStore::new(root.clone(), Arc::new(Mutex::new(0)));
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        client: &trusted_media_client(),
        url: &url,
        range: ByteRange::new(0, 5),
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };

    let error = download_chunk(&spec, &sink, &mut stats, &token)
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
