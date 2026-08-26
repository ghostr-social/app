mod range_fixture;
mod raw_http;

use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{ChunkSink, ChunkSpec};
use ghostr_engine::adaptive::{RetrievalRequest, WholeBodyContract, WholeFetchReason};
use ghostr_engine::host_stats::HostStats;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use range_fixture::download_chunk_throttled;

#[tokio::test]
async fn capped_whole_accepts_a_shorter_declared_body_and_finishes_at_eof() {
    let response =
        b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: 16\r\n\r\n0123456789abcdef";
    let (url, request) = raw_http::spawn_raw_server(response).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-capped-length");
    let store = range_fixture::store(root.clone());
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        requests: &client,
        url: &url,
        request: RetrievalRequest::FetchWhole {
            contract: WholeBodyContract::Capped { maximum_bytes: 32 },
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
    .expect("bounded whole download");
    request.await.expect("captured request");

    assert_eq!(result.bytes_written, 16);
    assert_eq!(result.total_bytes, Some(16));
    assert_eq!(
        store.total_len("clip").await.expect("valid test fixture"),
        Some(16)
    );
    let _ = std::fs::remove_dir_all(root);
}
