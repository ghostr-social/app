mod delivery_fixture;
mod raw_http;

use delivery_fixture::items::{focus_now, seed_range, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness_at;
use delivery_fixture::temp_directory;
use delivery_fixture::wait::{wait_for_file, wait_for_ranges};
use ghostr_engine::EngineParams;
use raw_http::spawn_split_response;
use std::time::Duration;

#[tokio::test]
async fn range_blind_origin_promotes_the_live_response_without_an_etag() {
    let root = temp_directory("ghostr-manager-range-blind");
    let probe = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: 16\r\nAccept-Ranges: bytes\r\nConnection: close\r\n\r\n";
    let prefix = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: 16\r\nConnection: close\r\n\r\n0123";
    let origin = spawn_split_response(probe, prefix, b"456789abcdef").await;
    let item = sized_item("aa11", &origin.url, 16, 1_000);
    let mut options = DeliveryOptions::default();
    options.params = EngineParams {
        chunk_bytes: 4,
        ..options.params
    };
    options.tuning.retry.transient_attempts = 1;
    let harness = start_harness_at(root, options);

    harness.handle.update_focus(focus_now(vec![item], 0, 5_000));
    origin.prefix_sent.await.expect("live prefix");
    tokio::time::timeout(
        Duration::from_millis(500),
        wait_for_ranges(&harness.store, "aa11", &[(0, 4)]),
    )
    .await
    .expect("promoted prefix must be exposed before response completion");
    origin.release.notify_one();
    origin.requests.await.expect("probe and promoted response");
    wait_for_file(&harness.root.join("aa11.video")).await;

    assert_eq!(
        std::fs::read(harness.root.join("aa11.video")).expect("completed bytes"),
        b"0123456789abcdef"
    );
    std::fs::remove_dir_all(&harness.root).expect("remove store");
}

#[tokio::test]
async fn nonzero_range_promotes_its_live_200_instead_of_restarting() {
    let root = temp_directory("ghostr-manager-seeded-range-blind");
    let probe = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: 16\r\nAccept-Ranges: bytes\r\nConnection: close\r\n\r\n";
    let prefix = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: 16\r\nConnection: close\r\n\r\n0123";
    let origin = spawn_split_response(probe, prefix, b"456789abcdef").await;
    let item = sized_item("bb22", &origin.url, 16, 1_000);
    let mut options = DeliveryOptions::default();
    options.params = EngineParams {
        chunk_bytes: 4,
        ..options.params
    };
    let harness = start_harness_at(root, options);
    seed_range(&harness.store, &item, 0, b"0123").await;

    harness.handle.update_focus(focus_now(vec![item], 0, 5_000));
    origin.prefix_sent.await.expect("live full-body prefix");
    let request = origin.body_request.await.expect("recorded body request");
    assert!(
        String::from_utf8_lossy(&request).contains("range: bytes=4-7"),
        "expected a nonzero probe: {}",
        String::from_utf8_lossy(&request)
    );
    origin.release.notify_one();
    origin.requests.await.expect("probe and one promoted body");
    wait_for_file(&harness.root.join("bb22.video")).await;
    assert_eq!(
        std::fs::read(harness.root.join("bb22.video")).unwrap(),
        b"0123456789abcdef"
    );
    std::fs::remove_dir_all(&harness.root).expect("remove store");
}
