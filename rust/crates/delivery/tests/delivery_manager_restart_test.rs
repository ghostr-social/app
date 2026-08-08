//! After a restart the manager resumes from the persisted range
//! manifest instead of re-downloading bytes already on disk.

mod delivery_fixture;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, hits, media_body, serve_recording};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness_at;
use delivery_fixture::temp_directory;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn delivery_manager_resumes_from_the_persisted_manifest() {
    let root = temp_directory("ghostr-delivery-restart");
    let earlier = PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    );
    earlier
        .write_range("aa11", 0, &media_body()[..8])
        .await
        .expect("seed the first half");

    let log = hit_log();
    let origin = serve_recording("origin", media_body(), log.clone()).await;
    let harness = start_harness_at(root, DeliveryOptions::default());
    harness.handle.update_focus(focus_now(
        vec![sized_item("aa11", &origin, 16, 1_000)],
        0,
        5_000,
    ));

    wait_for_ranges(&harness.store, "aa11", &[(0, 16)]).await;
    let recorded = hits(&log);
    assert!(
        recorded.iter().all(|hit| hit.starts_with("origin:GET:8-")),
        "only the missing tail may be fetched: {recorded:?}"
    );
    assert!(!recorded.is_empty());
    std::fs::remove_dir_all(&harness.root).ok();
}
