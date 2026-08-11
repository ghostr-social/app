//! Store pressure parks network work until capacity actually changes.
//! Already-playable bytes remain useful and a local refusal never spends
//! the source's retry budget.

mod delivery_fixture;

use delivery_fixture::full_disk::{discard, limits, paced_store};
use delivery_fixture::items::{focus_now, seed_range, sized_item};
use delivery_fixture::media::{hit_log, hits, media_body, serve_recording};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness_with_store;
use delivery_fixture::stats::wait_for;
use ghostr_engine::host_stats::host_of;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn delivery_parks_until_real_capacity_change_then_resumes_same_source() {
    let fixture = paced_store(
        "ghostr-delivery-full",
        limits(8, 0),
        1_000,
        Duration::from_secs(60),
    );
    std::fs::create_dir_all(&fixture.root).expect("store root");
    let log = hit_log();
    let origin = serve_recording("origin", media_body(), log.clone()).await;
    let item = sized_item("aa11", &origin, 16, 2_000);
    seed_range(&fixture.store, &item, 0, &media_body()[..8]).await;
    let root = fixture.root.clone();
    let mut options = DeliveryOptions::default();
    options.params.head_seconds = 1;
    options.tuning.retry.transient_attempts = 1;
    let harness = start_harness_with_store(Arc::new(fixture.store), root, options);
    harness.handle.update_focus(focus_now(vec![item], 0, 5_000));

    wait_for_refusal(&harness.store).await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    assert_eq!(
        hits(&log).len(),
        1,
        "unchanged capacity must park origin IO"
    );
    assert_eq!(harness.store.refusals(), 1, "one capacity decision");
    assert_range(&harness.store, 0..8).await;

    harness.store.set_storage_budget(16).await.unwrap();
    wait_for_complete(&harness.store).await;
    assert_eq!(hits(&log).len(), 2, "the same source resumes once");
    assert!(harness.posts.contains("aa11"));
    let host = host_of(&origin).unwrap();
    let stats = wait_for(&harness.root.join("host_stats.json"), |stats| {
        stats.host_throughput(&host).is_some()
    })
    .await;
    assert_eq!(
        stats.failure_ratio(&host),
        0.0,
        "local pressure is not network failure"
    );
    discard(&fixture.root);
}

async fn wait_for_refusal(store: &PartialRangeStore) {
    timeout(Duration::from_secs(2), async {
        while store.refusals() == 0 {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("first store refusal");
}

async fn wait_for_complete(store: &PartialRangeStore) {
    timeout(Duration::from_secs(2), async {
        while !has_range(store, &(0..16)).await {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("capacity wake resumes parked post");
}

async fn assert_range(store: &PartialRangeStore, wanted: std::ops::Range<u64>) {
    assert!(has_range(store, &wanted).await, "useful bytes stay present");
}

async fn has_range(store: &PartialRangeStore, wanted: &std::ops::Range<u64>) -> bool {
    let ranges = store.present_ranges("aa11").await.unwrap();
    ranges.len() == 1 && ranges.first() == Some(wanted)
}
