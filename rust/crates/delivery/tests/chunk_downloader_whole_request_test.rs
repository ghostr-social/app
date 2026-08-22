mod range_fixture;
mod raw_http;

use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{ChunkSink, ChunkSpec};
use ghostr_engine::adaptive::{RetrievalRequest, WholeBodyContract, WholeFetchReason};
use ghostr_engine::host_stats::HostStats;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use range_fixture::download_chunk_throttled;

#[tokio::test]
async fn whole_fetch_omits_range_headers_and_accepts_exact_200() {
    let response =
        b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: 16\r\n\r\n0123456789abcdef";
    let (url, request) = raw_http::spawn_raw_server(response).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-whole-request");
    let store = range_fixture::store(root.clone());
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        requests: &client,
        url: &url,
        request: RetrievalRequest::FetchWhole {
            contract: WholeBodyContract::Exact { expected_bytes: 16 },
            reason: WholeFetchReason::DirectCrossover,
        },
        continuation: None,
        priority: ghostr_engine::adaptive::PreemptionAuthority::Transition,
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };

    let result = download_chunk_throttled(
        &spec,
        &sink,
        range_fixture::context(&mut stats, &token, &range_fixture::network()),
    )
    .await
    .expect("whole download");
    let request =
        String::from_utf8(request.await.expect("captured request")).expect("request text");

    assert!(request.starts_with("GET /video.mp4 HTTP/1.1"));
    assert!(!request.to_ascii_lowercase().contains("range:"));
    assert!(request
        .to_ascii_lowercase()
        .contains("accept-encoding: identity"));
    assert_eq!(result.bytes_written, 16);
    assert_eq!(
        store.read_range("clip", 0..16).await.expect("read"),
        Some(b"0123456789abcdef".to_vec())
    );
    let _ = std::fs::remove_dir_all(root);
}
