mod range_fixture;

use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{ChunkSink, ChunkSpec};
use ghostr_engine::host_stats::{host_of, HostStats, OPTIMISTIC_THROUGHPUT_BPS};
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use range_fixture::download_chunk_throttled;

#[tokio::test]
async fn chunk_downloader_records_throughput_and_success_for_the_host() {
    let url = range_fixture::ranged::serve_ranged(range_fixture::body()).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-chunk-throughput");
    let store = range_fixture::store(root.clone());
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        client: &client,
        url: &url,
        range: ByteRange::new(0, 16),
        continuation: None,
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };

    download_chunk_throttled(&spec, &sink, &mut stats, &token, &range_fixture::network())
        .await
        .expect("chunk download");

    let host = host_of(&url).expect("fixture host");
    let throughput = stats.expected_throughput(&host);
    assert!(throughput > 0.0);
    assert_ne!(throughput, OPTIMISTIC_THROUGHPUT_BPS);
    assert_eq!(stats.failure_ratio(&host), 0.0);
    let _ = std::fs::remove_dir_all(root);
}
