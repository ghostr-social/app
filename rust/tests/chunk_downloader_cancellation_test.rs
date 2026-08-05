mod range_fixture;

use rust_lib_ghostr::engine::host_stats::HostStats;
use rust_lib_ghostr::engine::ByteRange;
use rust_lib_ghostr::video::chunk_cancel::{cancel_pair, CancelHandle};
use rust_lib_ghostr::video::chunk_downloader::{download_chunk, ChunkSink, ChunkSpec};
use rust_lib_ghostr::video::partial_range_store::PartialRangeStore;
use rust_lib_ghostr::video::transfer_timeouts::TransferTimeouts;
use std::time::Duration;

#[tokio::test]
async fn chunk_downloader_cancellation_mid_stream_keeps_the_partial_bytes() {
    let url = range_fixture::stall::serve_stalling(b"abcd".to_vec(), 8).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-chunk-cancel");
    let store = range_fixture::store(root.clone());
    let (handle, token) = cancel_pair();
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

    let (result, ()) = tokio::join!(
        download_chunk(&spec, &sink, &mut stats, &token),
        cancel_once_partial(&store, &handle),
    );

    let result = result.expect("cancelled download");
    assert!(result.cancelled);
    assert_eq!(result.bytes_written, 4);
    let ranges = store.present_ranges("clip").await.expect("ranges");
    assert!(is_head_only(&ranges));
    let _ = std::fs::remove_dir_all(root);
}

async fn cancel_once_partial(store: &PartialRangeStore, handle: &CancelHandle) {
    loop {
        let ranges = store.present_ranges("clip").await.expect("poll ranges");
        if is_head_only(&ranges) {
            handle.cancel();
            return;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

fn is_head_only(ranges: &[std::ops::Range<u64>]) -> bool {
    matches!(ranges, [range] if range.start == 0 && range.end == 4)
}
