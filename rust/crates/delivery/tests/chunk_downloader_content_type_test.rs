mod range_fixture;

use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{ChunkSink, ChunkSpec};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use range_fixture::download_chunk_throttled;

#[tokio::test]
async fn chunk_downloader_rejects_an_explicit_image_before_writing_bytes() {
    let body = range_fixture::body();
    let url = range_fixture::content_type::serve(Some("image/png"), body).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-chunk-image");
    let store = range_fixture::store(root.clone());
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        client: &client,
        url: &url,
        request: range_fixture::range_request(ByteRange::new(0, 16)),
        continuation: None,
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };

    let result =
        download_chunk_throttled(&spec, &sink, &mut stats, &token, &range_fixture::network()).await;

    assert!(result.is_err(), "an image response must not be downloaded");
    assert_eq!(store.present_ranges("clip").await.expect("ranges"), vec![]);
    std::fs::remove_dir_all(root).ok();
}
