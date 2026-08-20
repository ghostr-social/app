//! Debug bandwidth pacing must expose bounded progress even when the HTTP
//! client delivers a large response frame.

mod range_fixture;

use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::{ChunkSink, ChunkSpec};
use ghostr_delivery::debug::network::NetworkProfile;
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use range_fixture::download_chunk_throttled;
use std::time::Duration;

const TOTAL: u64 = 256 * 1_024;
const FIRST_PACED_WRITE: u64 = 16 * 1_024;
const PROGRESS_WATCHDOG: Duration = Duration::from_secs(10);

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
        request: range_fixture::range_request(ByteRange::new(0, TOTAL)),
        continuation: None,
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

    let download = download_chunk_throttled(&spec, &sink, &mut stats, &token, &network);
    tokio::pin!(download);
    let progress = tokio::time::timeout(PROGRESS_WATCHDOG, async {
        tokio::select! {
            result = &mut download => panic!("download completed before prefix: {result:?}"),
            () = wait_for_prefix(&store) => {}
        }
    });

    progress
        .await
        .expect("large response is stored incrementally");
    download.await.expect("chunk download");
    std::fs::remove_dir_all(root).ok();
}

async fn wait_for_prefix(store: &PartialRangeStore) {
    let notify = store.change_notifier();
    loop {
        let changed = notify.notified();
        let ranges = store.present_ranges("clip").await.expect("ranges");
        if ranges
            .iter()
            .any(|range| range.start == 0 && range.end >= FIRST_PACED_WRITE)
        {
            return;
        }
        changed.await;
    }
}
