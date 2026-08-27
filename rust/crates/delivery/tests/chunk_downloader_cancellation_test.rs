mod range_fixture;

use core::time::Duration;
use ghostr_delivery::chunk::cancel::{cancel_pair, CancelHandle};
use ghostr_delivery::chunk::downloader::{ChunkSink, ChunkSpec};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use range_fixture::download_chunk_throttled;

#[tokio::test]
async fn chunk_downloader_cancellation_mid_stream_keeps_the_partial_bytes() {
    let url = range_fixture::stall::serve_stalling(b"abcd".to_vec(), 8).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-chunk-cancel");
    let store = range_fixture::store(root.clone());
    let (handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        requests: &client,
        url: &url,
        request: range_fixture::range_request(ByteRange::new(0, 8)),
        attempt_profile: range_fixture::range_profile(8),
        continuation: None,
        priority: ghostr_engine::adaptive::PreemptionAuthority::Transition,
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };
    let network = range_fixture::network();

    let (result, ()) = tokio::join!(
        download_chunk_throttled(
            &spec,
            &sink,
            range_fixture::context(&mut stats, &token, &network)
        ),
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

fn is_head_only(ranges: &[core::ops::Range<u64>]) -> bool {
    matches!(ranges, [range] if range.start == 0 && range.end == 4)
}
