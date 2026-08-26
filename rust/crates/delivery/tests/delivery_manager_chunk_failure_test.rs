mod delivery_fixture;
mod raw_http;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use delivery_fixture::stats::wait_for;
use ghostr_engine::host_stats::host_of;
use raw_http::spawn_response_sequence;

#[tokio::test]
async fn failed_chunk_is_charged_to_the_origin_without_storing_bytes() {
    let probe = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: 16\r\nAccept-Ranges: bytes\r\nConnection: close\r\n\r\n";
    let failure =
        b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
    let (origin, requests) = spawn_response_sequence(vec![probe, failure]).await;
    let mut options = DeliveryOptions::default();
    options.tuning.retry.transient_attempts = 1;
    let harness = start_harness("ghostr-manager-chunk-failure", options);

    harness.handle.update_focus(focus_now(
        vec![sized_item("failed", &origin, 16, 1_000)],
        0,
        0,
    ));
    requests.await.expect("probe and failed chunk request");
    let host = host_of(&origin).expect("fixture host");
    let stats = wait_for(&harness.root.join("host_stats.json"), |stats| {
        stats.failure_ratio(&host) > 0.0
    })
    .await;

    assert!(stats.failure_ratio(&host) > 0.0);
    assert!(harness
        .store
        .present_ranges("failed")
        .await
        .expect("valid test fixture")
        .is_empty());
    std::fs::remove_dir_all(&harness.root).ok();
}
