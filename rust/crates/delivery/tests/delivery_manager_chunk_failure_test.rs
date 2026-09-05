mod delivery_fixture;
mod raw_http;

use core::time::Duration;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use delivery_fixture::stats::wait_for;
use delivery_fixture::DeliveryHarness;
use ghostr_engine::host_stats::host_of;
use raw_http::spawn_raw_server;
use tokio::task::JoinHandle;

#[tokio::test]
async fn failed_full_get_is_charged_to_the_origin_without_storing_bytes() {
    let failure =
        b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
    let (origin, request) = spawn_raw_server(failure).await;
    let mut options = DeliveryOptions::default();
    options.tuning.retry.transient_attempts = 1;
    let harness = start_harness("ghostr-manager-chunk-failure", options);

    harness.handle.update_focus(focus_now(
        vec![sized_item("failed", &origin, 16, 1_000)],
        0,
        0,
    ));
    assert_direct_full_get(request).await;
    assert_failure_recorded(&harness, &origin).await;
    std::fs::remove_dir_all(&harness.root).ok();
}

async fn assert_direct_full_get(request: JoinHandle<Vec<u8>>) {
    let request = tokio::time::timeout(Duration::from_secs(30), request)
        .await
        .expect("bounded direct full GET")
        .expect("valid test fixture");
    let request = String::from_utf8(request).expect("HTTP request text");
    assert!(
        request.starts_with("GET /video.mp4 HTTP/1.1\r\n"),
        "ordinary fallback uses a full GET"
    );
    assert!(
        !request.to_ascii_lowercase().contains("\r\nrange:"),
        "ordinary fallback omits Range"
    );
}

async fn assert_failure_recorded(harness: &DeliveryHarness, origin: &str) {
    let host = host_of(origin).expect("fixture host");
    let stats = wait_for(&harness.root.join("host_stats.json"), |stats| {
        stats.failure_ratio(&host) > 0.0
    })
    .await;

    assert!(
        stats.failure_ratio(&host) > 0.0,
        "origin failure updates its statistics"
    );
    assert!(
        harness
            .store
            .present_ranges("failed")
            .await
            .expect("valid test fixture")
            .is_empty(),
        "failed response does not create cached bytes"
    );
}
