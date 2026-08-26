mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, hits, media_body, serve_recording, HitLog};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness_with_store;
use delivery_fixture::temp_directory;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn unpersistable_total_length_does_not_stop_delivery_reconciliation() {
    let parent = temp_directory("ghostr-total-persistence-failure");
    std::fs::create_dir(&parent).expect("create test directory");
    let blocked_root = parent.join("blocked");
    std::fs::create_dir(&blocked_root).expect("create store root");
    let store = Arc::new(PartialRangeStore::with_capacity(
        blocked_root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    ));
    assert_eq!(store.total_len("aa11").await.expect("prime entry"), None);
    std::fs::remove_dir(&blocked_root).expect("remove store root");
    std::fs::write(&blocked_root, b"not a directory").expect("block store root");
    let log = hit_log();
    let origin = serve_recording("origin", media_body(), std::sync::Arc::clone(&log)).await;
    let harness = start_harness_with_store(store, blocked_root.clone(), DeliveryOptions::default());

    harness.handle.update_focus(focus_now(
        vec![sized_item("aa11", &origin, 16, 1_000)],
        0,
        0,
    ));

    wait_for_hit(&log).await;
    assert!(blocked_root.is_file(), "manifest write must have failed");
    std::fs::remove_dir_all(parent).expect("remove test directory");
}

async fn wait_for_hit(log: &HitLog) {
    tokio::time::timeout(Duration::from_secs(2), async {
        while hits(log).is_empty() {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("delivery should continue after manifest failure");
}
