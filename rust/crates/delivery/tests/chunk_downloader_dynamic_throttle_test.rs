//! A live bandwidth change must never charge already-persisted bytes again.

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
const DROP_AFTER: u64 = 160 * 1_024;
const QUANTUM: u64 = 16 * 1_024;
const POST_DROP_WATCHDOG: Duration = Duration::from_secs(1);

#[tokio::test]
async fn bandwidth_drop_paces_only_bytes_not_yet_delivered() {
    let url = range_fixture::ranged::serve_ranged(vec![7; TOTAL as usize]).await;
    let client = range_fixture::media_client();
    let root = range_fixture::temp_root("ghostr-dynamic-throttle");
    let store = range_fixture::store(root.clone());
    let network = range_fixture::network();
    network.update(profile(2_500));
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        requests: &client,
        url: &url,
        request: range_fixture::range_request(ByteRange::new(0, TOTAL)),
        continuation: None,
        priority: ghostr_engine::adaptive::PreemptionAuthority::Transition,
        timeouts: TransferTimeouts::default(),
    };
    let sink = ChunkSink {
        store: &store,
        key: "clip",
    };

    let (download, progress) = tokio::join!(
        download_chunk_throttled(
            &spec,
            &sink,
            range_fixture::context(&mut stats, &token, &network)
        ),
        verify_post_drop_progress(&store, &network),
    );

    download.expect("chunk download");
    progress.expect("next 700 kbps quantum arrives without retroactive debt");
    std::fs::remove_dir_all(root).ok();
}

async fn verify_post_drop_progress(
    store: &PartialRangeStore,
    network: &ghostr_delivery::debug::network::NetworkThrottle,
) -> Result<(), tokio::time::error::Elapsed> {
    wait_for_prefix(store, DROP_AFTER).await;
    network.update(profile(700));
    let first = prefix_end(store).await + QUANTUM;
    wait_for_prefix(store, first).await;
    let next = prefix_end(store).await + QUANTUM;
    // One 16 KiB quantum needs about 188 ms at 700 kbps. A one-second
    // watchdog tolerates a loaded CI host but still fails well before
    // charging the already-written 160 KiB again (about 1.9 seconds).
    tokio::time::timeout(POST_DROP_WATCHDOG, wait_for_prefix(store, next)).await
}

async fn wait_for_prefix(store: &PartialRangeStore, minimum: u64) {
    let notify = store.change_notifier();
    loop {
        let changed = notify.notified();
        if prefix_end(store).await >= minimum {
            return;
        }
        changed.await;
    }
}

async fn prefix_end(store: &PartialRangeStore) -> u64 {
    store
        .present_ranges("clip")
        .await
        .expect("ranges")
        .first()
        .map_or(0, |range| range.end)
}

fn profile(bandwidth_kbps: u64) -> NetworkProfile {
    NetworkProfile {
        bandwidth_kbps,
        ..NetworkProfile::default()
    }
}
