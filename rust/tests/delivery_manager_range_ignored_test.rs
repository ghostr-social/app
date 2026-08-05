mod support;

use rust_lib_ghostr::video::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use support::delivery::start_harness_at;
use support::delivery_items::{focus_now, sized_item};
use support::delivery_options::DeliveryOptions;
use support::delivery_wait::wait_not_servable;
use support::fixtures::temp_directory;
use support::http::spawn_raw_server;
use tokio::sync::Mutex;

#[tokio::test]
async fn range_blind_origin_cannot_erase_or_advance_a_resumed_download() {
    let root = temp_directory("ghostr-manager-range-blind");
    let earlier = PartialRangeStore::new(root.clone(), Arc::new(Mutex::new(0)));
    earlier
        .write_range("aa11", 0, b"01234567")
        .await
        .expect("seed head");
    let response = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: 16\r\nConnection: close\r\n\r\n0123456789abcdef";
    let (origin, request) = spawn_raw_server(response).await;
    let mut options = DeliveryOptions::default();
    options.tuning.retry.transient_attempts = 1;
    let harness = start_harness_at(root, options);

    harness.handle.update_focus(focus_now(
        vec![sized_item("aa11", &origin, 16, 1_000)],
        0,
        5_000,
    ));
    request.await.expect("range request");
    wait_not_servable(&harness.posts, "aa11").await;

    assert_eq!(
        harness.store.present_ranges("aa11").await.expect("ranges"),
        vec![0..8]
    );
    assert!(!harness.store.completed_path("aa11").exists());
    std::fs::remove_dir_all(&harness.root).expect("remove store");
}
