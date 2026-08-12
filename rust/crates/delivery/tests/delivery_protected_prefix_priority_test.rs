mod delivery_fixture;

use delivery_fixture::host_hol::SlowHost;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, hits, serve_recording, HitLog};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_engine::EngineParams;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::time::Duration;

const PROTECTED: [&str; 4] = ["current", "next-1", "next-2", "next-3"];

#[tokio::test]
async fn far_origin_never_runs_outside_the_protected_prefix() {
    let slow = SlowHost::serve().await;
    let far_hits = hit_log();
    let fast = serve_recording("far", vec![2; 64], far_hits.clone()).await;
    let harness = start_harness("ghostr-protected-prefix", options());
    harness
        .handle
        .update_focus(focus_now(items(&slow, &fast), 0, 0));
    tokio::time::timeout(Duration::from_secs(1), slow.wait_started())
        .await
        .expect("protected transfer starts");

    assert!(!far_get_within(&far_hits, Duration::from_millis(300)).await);
    assert!(!any_far_bytes(&harness.store).await);
    slow.release();
    slow.release();
    for post in PROTECTED {
        wait_for_ranges(&harness.store, post, &[(0, 64)]).await;
    }
    assert!(!far_get_within(&far_hits, Duration::from_millis(300)).await);
    assert!(!any_far_bytes(&harness.store).await);

    harness.handle.clear().await.unwrap();
    std::fs::remove_dir_all(&harness.root).ok();
}

fn items(slow: &SlowHost, fast: &str) -> Vec<ghostr_delivery::delivery_events::FocusItem> {
    let mut items: Vec<_> = PROTECTED
        .iter()
        .map(|id| sized_item(id, &slow.url(id), 64, 1_000))
        .collect();
    items.extend([
        sized_item("far-1", fast, 64, 1_000),
        sized_item("far-2", fast, 64, 1_000),
        sized_item("far-3", fast, 64, 1_000),
    ]);
    items
}

async fn far_get_within(log: &HitLog, duration: Duration) -> bool {
    tokio::time::timeout(duration, async {
        loop {
            if hits(log).iter().any(|hit| hit.starts_with("far:GET")) {
                return;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .is_ok()
}

async fn any_far_bytes(store: &PartialRangeStore) -> bool {
    for post in ["far-1", "far-2", "far-3"] {
        if !store.present_ranges(post).await.unwrap().is_empty() {
            return true;
        }
    }
    false
}

fn options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams {
            head_seconds: 8,
            head_cap_bytes: 64,
            chunk_bytes: 64,
            startable_target: 4,
            startable_window: 4,
            balanced_concurrency: 2,
            ..base_params()
        },
        ..DeliveryOptions::default()
    }
}
