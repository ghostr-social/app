//! One WARP planning event commits only its selected HEAD action.

mod delivery_fixture;
mod raw_http;

use core::time::Duration;
use delivery_fixture::items::{focus_now, unsized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use raw_http::spawn_stalled_headers;

#[tokio::test]
async fn unresolved_window_launches_only_the_selected_head() {
    let first = spawn_stalled_headers().await;
    let mut second = spawn_stalled_headers().await;
    let harness = start_harness("warp-selected-head", DeliveryOptions::default());

    harness.handle.update_focus(focus_now(
        vec![
            unsized_item("first", &first.url),
            unsized_item("second", &second.url),
        ],
        0,
        0,
    ));

    first
        .request_started
        .await
        .expect("selected HEAD must reach the first origin");
    assert!(
        tokio::time::timeout(Duration::from_millis(100), &mut second.request_started)
            .await
            .is_err(),
        "an unselected HEAD must not launch during the same planning event"
    );
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}
