//! After a restart the manager resumes from the persisted range
//! manifest instead of re-downloading bytes already on disk.

mod support;

use rust_lib_ghostr::video::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use support::delivery::{start_harness_at, DeliveryOptions};
use support::delivery_items::{focus_now, sized_item};
use support::delivery_media::{hit_log, hits, media_body, serve_recording};
use support::delivery_wait::wait_for_ranges;
use support::fixtures::temp_directory;
use tokio::sync::Mutex;

#[tokio::test]
async fn delivery_manager_resumes_from_the_persisted_manifest() {
    let root = temp_directory("ghostr-delivery-restart");
    let earlier = PartialRangeStore::new(root.clone(), Arc::new(Mutex::new(0)));
    earlier
        .write_range("aa11", 0, &media_body()[..8])
        .await
        .expect("seed the first half");

    let log = hit_log();
    let origin = serve_recording("origin", media_body(), log.clone()).await;
    let harness = start_harness_at(root, DeliveryOptions::default());
    harness
        .handle
        .update_focus(focus_now(vec![sized_item("aa11", &origin, 16, 1_000)], 0, 5_000));

    wait_for_ranges(&harness.store, "aa11", &[(0, 16)]).await;
    let recorded = hits(&log);
    assert!(
        recorded.iter().all(|hit| hit.starts_with("origin:GET:8-")),
        "only the missing tail may be fetched: {recorded:?}"
    );
    assert!(!recorded.is_empty());
    std::fs::remove_dir_all(&harness.root).ok();
}
