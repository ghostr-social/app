mod range_fixture;
mod raw_http;

use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{ChunkSink, ChunkSpec};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::representation::SourceGeneration;
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;

const DUPLICATE_RANGE: &[u8] = b"HTTP/1.1 206 Partial Content\r\n\
Content-Type: video/mp4\r\n\
Content-Length: 8\r\n\
Content-Range: bytes 8-15/16\r\n\
Content-Range: bytes 8-15/32\r\n\
ETag: \"fixture-media\"\r\n\
Connection: close\r\n\r\n89abcdef";

#[tokio::test]
async fn duplicate_content_range_cannot_extend_a_sparse_generation() {
    let (url, request) = raw_http::spawn_raw_server(DUPLICATE_RANGE).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("duplicate-content-range");
    let store = range_fixture::store(root.clone());
    store.write_range("clip", 0, b"01234567").await.unwrap();
    store.set_total_len("clip", 16).await.unwrap();
    let generation = SourceGeneration::try_new(&url, "\"fixture-media\"", 16).unwrap();
    let (_handle, token) = cancel_pair();
    let spec = ChunkSpec {
        client: &client,
        url: &url,
        request: range_fixture::range_request(ByteRange::new(8, 16)),
        continuation: Some(&generation),
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };

    let result = range_fixture::download_chunk_throttled(
        &spec,
        &sink,
        &mut HostStats::new(),
        &token,
        &range_fixture::network(),
    )
    .await;

    assert!(result.is_err(), "ambiguous 206 must fail before storage");
    assert_eq!(store.present_ranges("clip").await.unwrap(), vec![0..8]);
    let request = String::from_utf8(request.await.unwrap()).unwrap();
    assert!(request.contains("range: bytes=8-15"));
    assert!(request.contains("if-range: \"fixture-media\""));
    std::fs::remove_dir_all(root).ok();
}
