mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::paced_media;
use delivery_fixture::start_harness;
use ghostr_engine::host_stats::{host_of, HostStats};
use ghostr_engine::EngineParams;
use std::path::Path;

const TOTAL: u64 = 4 * 1024 * 1024;

#[tokio::test]
async fn manager_learns_throughput_before_the_response_reaches_eof() {
    let origin = paced_media::serve(
        TOTAL,
        Duration::from_millis(300),
        Duration::from_millis(600),
    )
    .await;
    let harness = start_harness("ghostr-live-progress-stats", options());
    harness.handle.update_focus(focus_now(
        vec![sized_item("current", &origin, TOTAL, 40_000)],
        0,
        0,
    ));

    let stats = wait_for_throughput(&harness.root.join("host_stats.json")).await;

    assert!(stats.overall_throughput().is_some());
    assert!(stats
        .host_throughput(&host_of(&origin).expect("valid test fixture"))
        .is_some());
    assert!(stats.overall_ttfb().expect("valid test fixture") >= Duration::from_millis(10));
    assert!(!harness
        .store
        .is_complete("current")
        .await
        .expect("valid test fixture"));
    let stored: u64 = harness
        .store
        .present_ranges("current")
        .await
        .expect("valid test fixture")
        .iter()
        .map(|range| range.end - range.start)
        .sum();
    assert!(stored > 0 && stored < TOTAL, "stored bytes: {stored}");
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}

async fn wait_for_throughput(path: &Path) -> HostStats {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if let Ok(json) = tokio::fs::read_to_string(path).await {
                let stats = HostStats::from_json(&json).expect("valid test fixture");
                if stats
                    .overall_throughput()
                    .is_some_and(|estimate| estimate.bytes_per_second() > 0.0)
                {
                    return stats;
                }
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("live throughput persisted before EOF")
}

fn options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams {
            chunk_bytes: 1024 * 1024,
            ..base_params()
        },
        ..DeliveryOptions::default()
    }
}
