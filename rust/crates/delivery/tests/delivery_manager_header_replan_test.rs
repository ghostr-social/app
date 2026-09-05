mod delivery_fixture;
mod raw_http;

use delivery_fixture::items::{focus_now, seed_range, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_engine::EngineParams;
use raw_http::spawn_split_response;

#[tokio::test]
async fn whole_response_publishes_total_and_readable_prefix_before_body_eof() {
    let head = b"HTTP/1.1 200 OK\r\nCache-Control: public, max-age=3600\r\nContent-Type: video/mp4\r\nContent-Length: 16\r\nAccept-Ranges: bytes\r\nConnection: close\r\n\r\n";
    let body = b"HTTP/1.1 200 OK\r\nCache-Control: public, max-age=3600\r\nContent-Type: video/mp4\r\nContent-Length: 16\r\nETag: \"same\"\r\n\r\n01";
    let origin = spawn_split_response(head, body, b"23456789abcdef").await;
    let item = sized_item("post", &origin.url, 16, 1_000);
    let mut options = DeliveryOptions::default();
    options.params = EngineParams {
        chunk_bytes: 4,
        ..options.params
    };
    let harness = start_harness("header-replan", options);

    let current = sized_item("current", &origin.url, 16, 1_000);
    seed_range(&harness.store, &current, 0, b"0123456789abcdef").await;
    harness
        .handle
        .update_focus(focus_now(vec![current, item], 0, 5_000));
    let request = origin.body_request.await.expect("first body request");
    assert!(request.starts_with(b"GET "), "body GET starts directly");
    origin.prefix_sent.await.expect("first response prefix");
    delivery_fixture::wait::wait_total_len(&harness.store, "post", 16).await;
    delivery_fixture::wait::wait_for_ranges(&harness.store, "post", &[(0, 2)]).await;

    origin.release.notify_one();
    origin.requests.await.expect("first response completion");
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(harness.root).ok();
}
