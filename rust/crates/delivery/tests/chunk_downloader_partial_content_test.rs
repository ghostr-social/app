mod range_fixture;

use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{ChunkSink, ChunkSpec};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use range_fixture::download_chunk_throttled;

#[tokio::test]
async fn chunk_downloader_streams_partial_content_into_the_store() {
    let url = range_fixture::ranged::serve_ranged(range_fixture::body()).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-chunk-206");
    let store = range_fixture::store(root.clone());
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        client: &client,
        url: &url,
        range: ByteRange::new(4, 12),
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

    assert_eq!(result.bytes_written, 8);
    assert!(result.accept_ranges);
    assert!(!result.cancelled);
    assert_eq!(result.total_bytes, Some(16));
    assert_eq!(
        store.present_ranges("clip").await.expect("ranges"),
        vec![4..12]
    );
    let read = store.read_range("clip", 4..12).await.expect("read");
    assert_eq!(read, Some(b"456789ab".to_vec()));
    let _ = std::fs::remove_dir_all(root);
}
