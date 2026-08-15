mod range_fixture;

use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{ChunkSink, ChunkSpec};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use range_fixture::download_chunk_throttled;

#[tokio::test]
async fn cancellation_before_admission_finishes_without_an_http_request() {
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-chunk-pre-cancel");
    let store = range_fixture::store(root.clone());
    let (handle, token) = cancel_pair();
    handle.cancel();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        client: &client,
        url: "not an http URL",
        range: ByteRange::new(0, 8),
        continuation: None,
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };

    let result =
        download_chunk_throttled(&spec, &sink, &mut stats, &token, &range_fixture::network())
            .await
            .expect("pre-cancelled download");

    assert!(result.cancelled);
    assert_eq!(result.bytes_written, 0);
    assert!(!result.accept_ranges);
    assert_eq!(result.total_bytes, None);
    assert!(store.present_ranges("clip").await.unwrap().is_empty());
    std::fs::remove_dir_all(root).ok();
}
