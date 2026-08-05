mod range_fixture;

use rust_lib_ghostr::engine::host_stats::HostStats;
use rust_lib_ghostr::engine::ByteRange;
use rust_lib_ghostr::video::chunk_cancel::cancel_pair;
use rust_lib_ghostr::video::chunk_downloader::{download_chunk, ChunkSink, ChunkSpec};
use rust_lib_ghostr::video::transfer_timeouts::TransferTimeouts;

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
        range: ByteRange::new(0, 16),
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };

    let result = download_chunk(&spec, &sink, &mut stats, &token).await;

    assert!(result.is_err(), "an image response must not be downloaded");
    assert_eq!(store.present_ranges("clip").await.expect("ranges"), vec![]);
    std::fs::remove_dir_all(root).ok();
}
