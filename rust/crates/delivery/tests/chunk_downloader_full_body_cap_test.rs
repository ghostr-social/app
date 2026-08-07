mod range_fixture;

use ghostr_engine::host_stats::HostStats;
use ghostr_engine::ByteRange;
use ghostr_delivery::chunk_cancel::cancel_pair;
use ghostr_delivery::chunk_downloader::{download_chunk, ChunkSink, ChunkSpec};
use ghostr_net::transfer_timeouts::TransferTimeouts;

#[tokio::test]
async fn chunk_downloader_caps_a_200_full_body_stream_at_the_grant() {
    let url = range_fixture::ranged::serve_range_blind(range_fixture::body()).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-chunk-200-cap");
    let store = range_fixture::store(root.clone());
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        client: &client,
        url: &url,
        range: ByteRange::new(0, 4),
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };

    let result = download_chunk(&spec, &sink, &mut stats, &token)
        .await
        .expect("chunk download");

    assert_eq!(result.bytes_written, 4);
    assert!(!result.accept_ranges);
    assert!(!result.cancelled);
    assert_eq!(result.total_bytes, Some(16));
    assert_eq!(
        store.present_ranges("clip").await.expect("ranges"),
        vec![0..4]
    );
    let read = store.read_range("clip", 0..4).await.expect("read");
    assert_eq!(read, Some(b"0123".to_vec()));
    let _ = std::fs::remove_dir_all(root);
}
