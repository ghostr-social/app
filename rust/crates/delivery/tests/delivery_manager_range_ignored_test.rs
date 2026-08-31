mod delivery_fixture;
mod raw_http;

use delivery_fixture::decision::wait_for_promotion;
use delivery_fixture::items::{focus_now, seed_range, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness_at;
use delivery_fixture::temp_directory;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_engine::EngineParams;
use raw_http::spawn_gated_split_response;

#[tokio::test]
async fn nonzero_range_promotes_its_live_200_instead_of_restarting() {
    let root = temp_directory("ghostr-manager-seeded-range-blind");
    let probe = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: 16\r\nAccept-Ranges: bytes\r\nConnection: close\r\n\r\n";
    let prefix = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: 16\r\nConnection: close\r\n\r\n0123";
    let origin = spawn_gated_split_response(probe, prefix, b"456789abcdef").await;
    let item = sized_item("bb22", &origin.url, 16, 1_000);
    let mut options = DeliveryOptions::default();
    options.params = EngineParams {
        chunk_bytes: 4,
        ..options.params
    };
    let harness = start_harness_at(root, options);
    seed_range(&harness.store, &item, 0, b"0123").await;

    harness.handle.update_focus(focus_now(vec![item], 0, 5_000));
    let request = origin.body_request.await.expect("recorded body request");
    assert!(
        String::from_utf8_lossy(&request).contains("range: bytes=4-7"),
        "expected a nonzero probe: {}",
        String::from_utf8_lossy(&request)
    );
    wait_for_promotion(&harness.handle).await;
    origin.release_headers.notify_one();
    origin.prefix_sent.await.expect("live full-body prefix");
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
