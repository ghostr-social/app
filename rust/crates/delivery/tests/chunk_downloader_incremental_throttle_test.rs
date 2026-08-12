//! Debug bandwidth pacing must expose bounded progress even when the HTTP
//! client delivers a large response frame.

mod range_fixture;

use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{download_chunk_throttled, ChunkSink, ChunkSpec};
use ghostr_delivery::debug::network::NetworkProfile;
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::time::Duration;

const TOTAL: u64 = 256 * 1_024;
const MID_TRANSFER_PREFIX: u64 = 144 * 1_024;

#[tokio::test]
async fn throttled_large_response_makes_incremental_store_progress() {
    let url = range_fixture::ranged::serve_ranged(vec![7; TOTAL as usize]).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-incremental-throttle");
    let store = range_fixture::store(root.clone());
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        client: &client,
        url: &url,
        range: ByteRange::new(0, TOTAL),
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };
    let network = range_fixture::network();
    network.update(NetworkProfile {
        bandwidth_kbps: 2_500,
        ..NetworkProfile::default()
    });

    let (download, progress) = tokio::join!(
        download_chunk_throttled(&spec, &sink, &mut stats, &token, &network),
        tokio::time::timeout(Duration::from_millis(650), wait_for_prefix(&store)),
    );

    progress.expect("large response is stored incrementally");
    download.expect("chunk download");
    std::fs::remove_dir_all(root).ok();
}

async fn wait_for_prefix(store: &PartialRangeStore) {
    let notify = store.change_notifier();
    loop {
        let changed = notify.notified();
        let ranges = store.present_ranges("clip").await.expect("ranges");
        if ranges
            .iter()
            .any(|range| range.start == 0 && range.end >= MID_TRANSFER_PREFIX)
        {
            return;
        }
        changed.await;
    }
}
