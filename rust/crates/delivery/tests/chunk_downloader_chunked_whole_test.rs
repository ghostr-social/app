mod range_fixture;
mod raw_http;

use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{ChunkSink, ChunkSpec};
use ghostr_engine::adaptive::{RetrievalRequest, WholeBodyContract, WholeFetchReason};
use ghostr_engine::host_stats::HostStats;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use range_fixture::download_chunk_throttled;

#[tokio::test]
async fn capped_chunked_whole_discovers_its_exact_length_at_eof() {
    let response = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nTransfer-Encoding: chunked\r\n\r\n8\r\n01234567\r\n8\r\n89abcdef\r\n0\r\n\r\n";
    let (url, request) = raw_http::spawn_raw_server(response).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-chunked-whole");
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
    .expect("chunked whole download");
    request.await.expect("captured request");

    assert_eq!(result.bytes_written, 16);
    assert_eq!(result.total_bytes, Some(16));
    assert_eq!(store.total_len("clip").await.expect("total"), Some(16));
    assert_eq!(
        store.read_range("clip", 0..16).await.expect("read"),
        Some(b"0123456789abcdef".to_vec())
    );
    let _ = std::fs::remove_dir_all(root);
}
