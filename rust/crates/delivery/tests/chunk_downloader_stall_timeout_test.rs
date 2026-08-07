mod range_fixture;

use ghostr_engine::host_stats::{host_of, HostStats};
use ghostr_engine::ByteRange;
use ghostr_delivery::chunk_cancel::cancel_pair;
use ghostr_delivery::chunk_downloader::{download_chunk, ChunkSink, ChunkSpec};
use ghostr_net::transfer_timeouts::TransferTimeouts;

#[tokio::test(start_paused = true)]
async fn chunk_downloader_times_out_on_a_stalled_transfer_and_records_a_failure() {
    let url = range_fixture::stall::serve_stalling(b"ab".to_vec(), 8).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-chunk-stall");
    let store = range_fixture::store(root.clone());
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        client: &client,
        url: &url,
        range: ByteRange::new(0, 8),
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };

    let result = download_chunk(&spec, &sink, &mut stats, &token).await;

    assert!(result.is_err());
    let host = host_of(&url).expect("fixture host");
    assert!(stats.failure_ratio(&host) > 0.0);
    let _ = std::fs::remove_dir_all(root);
}
