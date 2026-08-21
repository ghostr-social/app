mod range_fixture;
mod raw_http;

use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{ChunkSink, ChunkSpec};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;

const CODED_RANGE: &[u8] = b"HTTP/1.1 206 Partial Content\r\n\
Content-Type: video/mp4\r\n\
Content-Length: 5\r\n\
Content-Range: bytes 0-4/5\r\n\
ETag: \"coded\"\r\n\
Content-Encoding: identity\r\n\
Content-Encoding: gzip\r\n\
Connection: close\r\n\r\nvideo";

#[tokio::test]
async fn chunk_downloader_rejects_coded_bytes_hidden_after_identity() {
    let (url, request) = raw_http::spawn_raw_server(CODED_RANGE).await;
    let root = range_fixture::temp_root("chunk-coded-response");
    let store = range_fixture::store(root.clone());
    let client = range_fixture::media_client();
    let (_handle, token) = cancel_pair();
    let spec = ChunkSpec {
        requests: &client,
        url: &url,
        request: range_fixture::range_request(ByteRange::new(0, 5)),
        continuation: None,
        priority: ghostr_engine::adaptive::PreemptionAuthority::Transition,
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };

    range_fixture::download_chunk_throttled(
        &spec,
        &sink,
        range_fixture::context(&mut HostStats::new(), &token, &range_fixture::network()),
    )
    .await
    .expect_err("coded bytes must not enter the sparse store");

    assert!(store.present_ranges("clip").await.unwrap().is_empty());
    request.await.expect("upstream request");
    std::fs::remove_dir_all(root).ok();
}
