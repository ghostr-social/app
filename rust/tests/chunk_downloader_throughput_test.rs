mod range_fixture;

use rust_lib_ghostr::engine::host_stats::{host_of, HostStats, OPTIMISTIC_THROUGHPUT_BPS};
use rust_lib_ghostr::engine::ByteRange;
use rust_lib_ghostr::video::chunk_cancel::cancel_pair;
use rust_lib_ghostr::video::chunk_downloader::{download_chunk, ChunkSink, ChunkSpec};
use rust_lib_ghostr::video::transfer_timeouts::TransferTimeouts;

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
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };

    download_chunk(&spec, &sink, &mut stats, &token)
        .await
        .expect("chunk download");

    let host = host_of(&url).expect("fixture host");
    let throughput = stats.expected_throughput(&host);
    assert!(throughput > 0.0);
    assert_ne!(throughput, OPTIMISTIC_THROUGHPUT_BPS);
    assert_eq!(stats.failure_ratio(&host), 0.0);
    let _ = std::fs::remove_dir_all(root);
}
