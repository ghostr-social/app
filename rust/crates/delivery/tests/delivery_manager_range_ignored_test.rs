mod delivery_fixture;
mod raw_http;

use delivery_fixture::items::{focus_now, seed_range, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness_at;
use delivery_fixture::temp_directory;
use delivery_fixture::wait::wait_not_servable;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use raw_http::spawn_response_sequence;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn range_blind_origin_cannot_erase_or_advance_a_resumed_download() {
    let root = temp_directory("ghostr-manager-range-blind");
    let probe = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: 16\r\nAccept-Ranges: bytes\r\nConnection: close\r\n\r\n";
    let ignored = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: 16\r\nConnection: close\r\n\r\n0123456789abcdef";
    let (origin, requests) = spawn_response_sequence(vec![probe, ignored]).await;
    let item = sized_item("aa11", &origin, 16, 1_000);
    let earlier = PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    );
    seed_range(&earlier, &item, 0, b"01234567").await;
    let mut options = DeliveryOptions::default();
    options.tuning.retry.transient_attempts = 1;
    let harness = start_harness_at(root, options);

    harness.handle.update_focus(focus_now(vec![item], 0, 5_000));
    requests.await.expect("probe and range request");
    wait_not_servable(&harness.posts, "aa11").await;

    assert_eq!(
        harness.store.present_ranges("aa11").await.expect("ranges"),
        vec![0..8]
    );
    assert!(!harness.root.join("aa11.video").exists());
    std::fs::remove_dir_all(&harness.root).expect("remove store");
}
