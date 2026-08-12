mod delivery_fixture;

use delivery_fixture::items::{focus_now, unsized_item};
use delivery_fixture::media::{hit_log, hits, media_body, serve_recording, HitLog};
use delivery_fixture::probe_gate::ProbeGate;
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_total_len;
use std::time::Duration;

const CRITICAL: [&str; 4] = ["current", "next-1", "next-2", "next-3"];
const FAR: [&str; 3] = ["far-1", "far-2", "far-3"];

#[tokio::test]
async fn metadata_probes_wait_outside_the_protected_prefix() {
    let critical = ProbeGate::serve("next-3").await;
    let far_hits = hit_log();
    let far = serve_recording("far", media_body(), far_hits.clone()).await;
    let harness = start_harness("ghostr-probe-prefix", Default::default());
    harness
        .handle
        .update_focus(focus_now(items(&critical, &far), 0, 0));

    critical.wait_blocked().await;
    for post in &CRITICAL[..3] {
        wait_total_len(&harness.store, post, 64).await;
    }
    let premature = far_probe_within(&far_hits, Duration::from_millis(300)).await;
    harness
        .handle
        .update_focus(focus_now(items(&critical, &far), 1, 0));
    let after_advance = far_probe_within(&far_hits, Duration::from_secs(2)).await;

    harness.handle.clear().await.unwrap();
    std::fs::remove_dir_all(&harness.root).ok();
    assert!(!premature, "far metadata IO started before focus advanced");
    assert!(after_advance, "newly protected metadata was not probed");
}

fn items(critical: &ProbeGate, far: &str) -> Vec<ghostr_delivery::delivery_events::FocusItem> {
    let mut items: Vec<_> = CRITICAL
        .iter()
        .map(|id| unsized_item(id, &critical.url(id)))
        .collect();
    items.extend(FAR.iter().map(|id| unsized_item(id, far)));
    items
}

async fn far_probe_within(log: &HitLog, duration: Duration) -> bool {
    tokio::time::timeout(duration, async {
        loop {
            if hits(log).iter().any(|hit| hit.starts_with("far:")) {
                return;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .is_ok()
}
