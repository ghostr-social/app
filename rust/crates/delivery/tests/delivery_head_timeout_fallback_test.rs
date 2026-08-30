//! A stalled advisory HEAD must not consume the current video's startup window.

mod delivery_fixture;
mod delivery_head_timeout_origin;

use core::num::NonZeroUsize;
use core::time::Duration;
use delivery_fixture::items::{focus_now, unsized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;

#[tokio::test]
async fn stalled_head_yields_to_body_before_startup_deadline() {
    let origin = delivery_head_timeout_origin::serve().await;
    let mut options = DeliveryOptions::default();
    options.tuning.max_requests_per_authority = Some(NonZeroUsize::MIN);
    let harness = start_harness("head-timeout-fallback", options);
    harness
        .handle
        .update_focus(focus_now(vec![unsized_item("current", &origin.url)], 0, 0));

    let requests = tokio::time::timeout(Duration::from_secs(4), origin.requests)
        .await
        .expect("stalled HEAD must yield to a playable body request")
        .expect("origin task");
    assert!(requests.head.starts_with(b"HEAD "));
    assert!(requests.body.starts_with(b"GET "));

    harness.handle.clear().await.expect("clear delivery");
    std::fs::remove_dir_all(&harness.root).ok();
}
