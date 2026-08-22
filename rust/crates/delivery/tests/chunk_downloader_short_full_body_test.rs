mod range_fixture;

use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{ChunkSink, ChunkSpec};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use range_fixture::download_chunk_throttled;

#[tokio::test]
async fn short_200_body_completes_a_larger_speculative_grant() {
    let bytes = range_fixture::body();
    let url = range_fixture::ranged::serve_range_blind(bytes.clone()).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-chunk-short-200");
    let store = range_fixture::store(root.clone());
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        requests: &client,
        url: &url,
        request: range_fixture::range_request(ByteRange::new(0, 256 * 1024)),
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
    .expect("short full-body download");

    assert_eq!(result.bytes_written, bytes.len() as u64);
    assert_eq!(result.total_bytes, Some(bytes.len() as u64));
    assert_eq!(store.read_range("clip", 0..16).await.unwrap(), Some(bytes));
    let _ = std::fs::remove_dir_all(root);
}
