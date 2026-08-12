mod range_fixture;

use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{ChunkSink, ChunkSpec};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use range_fixture::download_chunk_throttled;

#[tokio::test]
async fn chunk_downloader_writes_nothing_when_the_server_ignores_a_nonzero_range() {
    let url = range_fixture::ranged::serve_range_blind(range_fixture::body()).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-chunk-range-ignored");
    let store = range_fixture::store(root.clone());
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        client: &client,
        url: &url,
        range: ByteRange::new(8, 16),
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };

    let result =
        download_chunk_throttled(&spec, &sink, &mut stats, &token, &range_fixture::network())
            .await
            .expect("chunk download");

    assert_eq!(result.bytes_written, 0);
    assert!(!result.accept_ranges);
    assert!(!result.cancelled);
    assert_eq!(result.total_bytes, Some(16));
    assert!(store
        .present_ranges("clip")
        .await
        .expect("ranges")
        .is_empty());
    let _ = std::fs::remove_dir_all(root);
}
