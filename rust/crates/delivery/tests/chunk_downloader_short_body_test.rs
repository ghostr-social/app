mod range_fixture;

use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{ChunkSink, ChunkSpec};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use range_fixture::download_chunk_throttled;

#[tokio::test]
async fn a_body_shorter_than_its_advertised_range_is_rejected() {
    let url = range_fixture::short::serve_short_partial().await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-chunk-short");
    let store = range_fixture::store(root.clone());
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        client: &client,
        url: &url,
        range: ByteRange::new(4, 12),
        continuation: None,
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };

    let result =
        download_chunk_throttled(&spec, &sink, &mut stats, &token, &range_fixture::network()).await;

    assert!(result.is_err(), "short 206 must not count as success");
    std::fs::remove_dir_all(root).ok();
}
