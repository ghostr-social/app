mod delivery_fixture;

use delivery_fixture::host_hol::SlowHost;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, hits, serve_recording};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::start_harness;
use ghostr_delivery::debug::network::NetworkProfile;
use ghostr_engine::EngineParams;
use std::time::Duration;

#[tokio::test]
async fn a_waiter_on_one_host_cannot_occupy_a_healthy_hosts_slot() {
    let slow = SlowHost::serve().await;
    let fast_hits = hit_log();
    let fast = serve_recording("fast", vec![2; 64], fast_hits.clone()).await;
    let harness = start_harness("ghostr-host-hol", options());
    harness.network.update(NetworkProfile {
        bandwidth_kbps: 0,
        latency_ms: 0,
        max_connections_per_host: 1,
    });
    harness.handle.update_focus(focus_now(
        vec![
            sized_item("slow-a", &slow.url("a"), 64, 8_000),
            sized_item("slow-b", &slow.url("b"), 64, 8_000),
            sized_item("fast", &fast, 64, 8_000),
        ],
        0,
        0,
    ));
    tokio::time::timeout(Duration::from_secs(1), slow.wait_started())
        .await
        .expect("slow transfer starts");

    tokio::time::timeout(Duration::from_millis(300), async {
        while !hits(&fast_hits)
            .iter()
            .any(|hit| hit.starts_with("fast:GET"))
        {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("healthy host uses the free global slot");

    slow.release();
    harness.handle.clear().await.unwrap();
    std::fs::remove_dir_all(&harness.root).ok();
}

fn options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams {
            balanced_concurrency: 2,
            chunk_bytes: 8,
            ..base_params()
        },
        ..DeliveryOptions::default()
    }
}
