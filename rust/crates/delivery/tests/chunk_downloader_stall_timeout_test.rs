mod range_fixture;

use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{ChunkSink, ChunkSpec};
use ghostr_engine::host_stats::{host_of, HostStats};
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use range_fixture::download_chunk_throttled;

#[tokio::test(start_paused = true)]
async fn chunk_downloader_times_out_on_a_stalled_transfer_and_records_a_failure() {
    let url = range_fixture::stall::serve_stalling(b"ab".to_vec(), 8).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-chunk-stall");
    let store = range_fixture::store(root.clone());
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        requests: &client,
        url: &url,
        request: range_fixture::range_request(ByteRange::new(0, 8)),
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
    .await;

    assert!(result.is_err());
    let host = host_of(&url).expect("fixture host");
    assert!(stats.failure_ratio(&host) > 0.0);
    let _ = std::fs::remove_dir_all(root);
}
