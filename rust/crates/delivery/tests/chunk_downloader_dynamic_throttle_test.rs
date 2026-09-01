//! A live bandwidth change must never charge already-delivered bytes again.

mod range_fixture;
#[path = "chunk_downloader_dynamic_throttle_test/support.rs"]
mod support;

use core::time::Duration;
use ghostr_delivery::chunk::cancel::cancel_pair;
use ghostr_delivery::chunk::downloader::ChunkSpec;
use ghostr_delivery::debug::network::NetworkProfile;
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::ByteRange;
use ghostr_net::transfer_timeouts::TransferTimeouts;
use support::{download, DownloadInput, ProgressSink};

const TOTAL: u64 = 256 * 1_024;
const DROP_AFTER: u64 = 160 * 1_024;
const QUANTUM: u64 = 16 * 1_024;
const POST_DROP_WATCHDOG: Duration = Duration::from_secs(1);

#[tokio::test]
async fn bandwidth_drop_paces_only_bytes_not_yet_delivered() {
    let url = range_fixture::ranged::serve_ranged(vec![7; TOTAL as usize]).await;
    let client = range_fixture::media_client();
    let sink = ProgressSink::default();
    let network = range_fixture::network();
    network.update(profile(2_500));
    let (_handle, token) = cancel_pair();
    let mut stats = HostStats::new();
    let spec = ChunkSpec {
        requests: &client,
        url: &url,
        request: range_fixture::range_request(ByteRange::new(0, TOTAL)),
        attempt_profile: range_fixture::range_profile(TOTAL),
        continuation: None,
        priority: ghostr_engine::adaptive::PreemptionAuthority::Transition,
        timeouts: TransferTimeouts::default(),
    };
    let (download, progress) = tokio::join!(
        download(DownloadInput {
            spec: &spec,
            sink: &sink,
            stats: &mut stats,
            cancel: &token,
            network: &network,
        }),
        verify_post_drop_progress(&sink, &network),
    );

    let result = download.expect("chunk download");
    assert_eq!(result.bytes_written, TOTAL);
    assert!(!result.cancelled);
    progress.expect("next 700 kbps quantum arrives without retroactive debt");
}

async fn verify_post_drop_progress(
    sink: &ProgressSink,
    network: &ghostr_delivery::debug::network::NetworkThrottle,
) -> Result<(), tokio::time::error::Elapsed> {
    sink.wait_for_prefix(DROP_AFTER).await;
    network.update(profile(700));
    let first = sink.prefix_end() + QUANTUM;
    sink.wait_for_prefix(first).await;
    let next = sink.prefix_end() + QUANTUM;
    // One 16 KiB quantum needs about 188 ms at 700 kbps. A one-second
    // watchdog tolerates a loaded CI host but still fails well before
    // charging the already-written 160 KiB again (about 1.9 seconds).
    tokio::time::timeout(POST_DROP_WATCHDOG, sink.wait_for_prefix(next)).await
}

fn profile(bandwidth_kbps: u64) -> NetworkProfile {
    NetworkProfile {
        bandwidth_kbps,
        ..NetworkProfile::default()
    }
}
