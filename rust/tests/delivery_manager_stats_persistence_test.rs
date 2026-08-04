//! Transfer outcomes are persisted as a host-stats JSON snapshot in
//! the cache directory and reloaded on the next start.

mod support;

use rust_lib_ghostr::engine::host_stats::HostStats;
use support::delivery::{start_harness, DeliveryOptions};
use support::delivery_items::{focus_now, sized_item};
use support::delivery_media::{hit_log, media_body, serve_recording};
use support::delivery_wait::{wait_for_file, wait_for_ranges};

#[tokio::test]
async fn delivery_manager_persists_host_stats_snapshots() {
    let origin = serve_recording("origin", media_body(), hit_log()).await;
    let harness = start_harness("ghostr-delivery-stats", DeliveryOptions::default());

    harness
        .handle
        .update_focus(focus_now(vec![sized_item("aa11", &origin, 16, 1_000)], 0, 0));

    wait_for_ranges(&harness.store, "aa11", &[(0, 16)]).await;
    let path = harness.root.join("host_stats.json");
    wait_for_file(&path).await;
    let json = tokio::fs::read_to_string(&path).await.expect("snapshot");
    let stats = HostStats::from_json(&json).expect("valid snapshot");
    let host = origin
        .strip_prefix("http://")
        .and_then(|rest| rest.split('/').next())
        .expect("fixture host");
    assert!(stats.expected_ttfb_ms(host).is_some() || stats.failure_ratio(host) == 0.0);
    assert!(stats.expected_throughput(host) > 0.0);
    std::fs::remove_dir_all(&harness.root).ok();
}
