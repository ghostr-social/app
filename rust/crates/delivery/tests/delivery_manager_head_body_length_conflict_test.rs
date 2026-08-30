mod delivery_fixture;
mod raw_http;

use delivery_fixture::head_window::serve_visible_current;
use delivery_fixture::items::{focus_now, unsized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use delivery_fixture::wait::{wait_for_ranges, wait_total_len};
use raw_http::spawn_response_sequence;

#[tokio::test]
async fn advisory_head_length_cannot_poison_coherent_body_extent() {
    let head = b"HTTP/1.1 200 OK\r\nContent-Type: video/mp4\r\nContent-Length: 8\r\nAccept-Ranges: bytes\r\nConnection: close\r\n\r\n";
    let body = b"HTTP/1.1 206 Partial Content\r\nContent-Type: video/mp4\r\nContent-Length: 8\r\nContent-Range: bytes 0-7/16\r\nETag: \"generation\"\r\nConnection: close\r\n\r\n01234567";
    let (url, requests) = spawn_response_sequence(vec![head, body]).await;
    let harness = start_harness("head-body-length-conflict", DeliveryOptions::default());
    let current = serve_visible_current().await;

    harness.handle.update_focus(focus_now(
        vec![current.item(), unsized_item("post", &url)],
        0,
        0,
    ));
    current.assert_get_without_head().await;
    requests.await.expect("valid test fixture");
    wait_for_ranges(&harness.store, "post", &[(0, 8)]).await;
    wait_total_len(&harness.store, "post", 16).await;

    assert_eq!(
        harness
            .store
            .total_len("post")
            .await
            .expect("valid test fixture"),
        Some(16)
    );
    std::fs::remove_dir_all(harness.root).ok();
}
