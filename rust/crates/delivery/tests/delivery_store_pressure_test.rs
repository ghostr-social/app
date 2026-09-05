//! A knowably full store plans no origin work at all — no byte is
//! bought just to be refused — and a real capacity change replans and
//! resumes the same source. Already-playable bytes remain useful and a
//! local store limit never spends the source's retry budget. (Actual
//! filesystem refusals still park via the pressure path; that fallback
//! covers disk state the configured budget cannot see.)

mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::full_disk::{discard, limits, paced_store};
use delivery_fixture::items::{focus_now, seed_range, sized_item};
use delivery_fixture::media::{hit_log, hits, media_body, serve_recording};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness_with_store;
use delivery_fixture::stats::wait_for;
use ghostr_engine::host_stats::host_of;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use tokio::time::timeout;

#[tokio::test]
async fn delivery_waits_for_real_capacity_change_then_resumes_same_source() {
    let fixture = paced_store(
        "ghostr-delivery-full",
        limits(8, 0),
        1_000,
        Duration::from_secs(60),
    );
    std::fs::create_dir_all(&fixture.root).expect("store root");
    let log = hit_log();
    let origin = serve_recording("origin", media_body(), std::sync::Arc::clone(&log)).await;
    let item = sized_item("aa11", &origin, 16, 2_000);
    seed_range(&fixture.store, &item, 0, &media_body()[..8]).await;
    let root = fixture.root.clone();
    let mut options = DeliveryOptions::default();
    options.tuning.retry.transient_attempts = 1;
    let harness = start_harness_with_store(Arc::new(fixture.store), root, options);
    harness.handle.update_focus(focus_now(vec![item], 0, 5_000));

    tokio::time::sleep(Duration::from_millis(150)).await;

    let get_count = || {
        hits(&log)
            .iter()
            .filter(|hit| hit.contains(":GET:"))
            .count()
    };
    assert_eq!(get_count(), 0, "a knowably full store buys no origin bytes");
    assert_eq!(
        harness.store.refusals(),
        0,
        "no write is attempted just to be refused"
    );
    assert_range(&harness.store, 0..8).await;

    harness
        .store
        .set_storage_budget(24)
        .await
        .expect("valid test fixture");
    wait_for_complete(&harness.store).await;
    assert_eq!(get_count(), 1, "the same source resumes once");
    assert!(harness.posts.contains("aa11"));
    let host = host_of(&origin).expect("valid test fixture");
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

async fn wait_for_complete(store: &PartialRangeStore) {
    timeout(Duration::from_secs(2), async {
        while !has_range(store, &(0..16)).await {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("capacity wake resumes parked post");
}

async fn assert_range(store: &PartialRangeStore, wanted: core::ops::Range<u64>) {
    assert!(has_range(store, &wanted).await, "useful bytes stay present");
}

async fn has_range(store: &PartialRangeStore, wanted: &core::ops::Range<u64>) -> bool {
    let ranges = store
        .present_ranges("aa11")
        .await
        .expect("valid test fixture");
    ranges.len() == 1 && ranges.first() == Some(wanted)
}
