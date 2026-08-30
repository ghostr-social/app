//! Visible bootstrap bytes and future metadata can launch independently.

mod delivery_fixture;
mod raw_http;

use delivery_fixture::head_window::serve_visible_current;
use delivery_fixture::items::{focus_now, unsized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use raw_http::spawn_raw_server;

const HEAD_RESPONSE: &[u8] =
    b"HTTP/1.1 200 OK\r\nContent-Length: 8\r\nAccept-Ranges: bytes\r\nConnection: close\r\n\r\n";

#[tokio::test]
async fn unresolved_window_launches_current_get_and_future_head() {
    let current = serve_visible_current().await;
    let (future_url, future_request) = spawn_raw_server(HEAD_RESPONSE).await;
    let harness = start_harness("warp-selected-head", DeliveryOptions::default());

    harness.handle.update_focus(focus_now(
        vec![current.item(), unsized_item("future", &future_url)],
        0,
        0,
    ));

    current.assert_get_without_head().await;
    let request = future_request.await.expect("future request");
    assert!(request.starts_with(b"HEAD "));
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}
