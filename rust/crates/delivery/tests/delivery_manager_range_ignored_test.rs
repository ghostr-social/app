mod delivery_fixture;
mod raw_http;

use delivery_fixture::items::{focus_now, seed_range, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness_at;
use delivery_fixture::temp_directory;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_engine::EngineParams;
use raw_http::spawn_gated_split_response;

#[tokio::test]
async fn validatorless_whole_fallback_keeps_seeded_bytes_until_complete() {
    let root = temp_directory("ghostr-manager-seeded-range-blind");
    let probe = b"HTTP/1.1 200 OK\r\nCache-Control: public, max-age=3600\r\nContent-Type: video/mp4\r\nContent-Length: 16\r\nAccept-Ranges: bytes\r\nConnection: close\r\n\r\n";
    let prefix = b"HTTP/1.1 200 OK\r\nCache-Control: public, max-age=3600\r\nContent-Type: video/mp4\r\nContent-Length: 16\r\nConnection: close\r\n\r\n0123";
    let origin = spawn_gated_split_response(probe, prefix, b"456789abcdef").await;
    let item = sized_item("bb22", &origin.url, 16, 1_000);
    let mut options = DeliveryOptions::default();
    options.params = EngineParams {
        chunk_bytes: 4,
        ..options.params
    };
    let harness = start_harness_at(root, options);
    seed_range(&harness.store, &item, 0, b"0123").await;

    let current = sized_item("current", &origin.url, 16, 1_000);
    seed_range(&harness.store, &current, 0, b"0123456789abcdef").await;
    let items = vec![current, item];
    harness
        .handle
        .update_focus(focus_now(items.clone(), 0, 5_000));
    let request = origin.body_request.await.expect("recorded body request");
    assert!(request.starts_with(b"GET "), "whole fallback starts");
    harness.handle.update_focus(focus_now(items, 1, 5_000));
    origin.release_headers.notify_one();
    origin.prefix_sent.await.expect("live full-body prefix");
    assert_eq!(
        harness
            .store
            .read_range("bb22", 0..4)
            .await
            .expect("seeded bytes"),
        Some(b"0123".to_vec())
    );
    origin.release.notify_one();
    origin.requests.await.expect("probe and one promoted body");
    wait_for_ranges(&harness.store, "bb22", &[(0, 16)]).await;
    assert_eq!(
        harness.store.read_range("bb22", 0..16).await.expect("read"),
        Some(b"0123456789abcdef".to_vec())
    );
    assert!(
        !harness.root.join("bb22.video").exists(),
        "validatorless bytes must not replace the seeded generation"
    );
    std::fs::remove_dir_all(&harness.root).expect("remove store");
}
