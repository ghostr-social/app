//! Redirect admission consumes the same advisory HEAD usefulness budget.

mod delivery_fixture;
mod delivery_redirect_admission_timeout_origin;

use core::num::NonZeroUsize;
use core::time::Duration;
use delivery_fixture::head_window::serve_visible_current;
use delivery_fixture::items::{focus_now, unsized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::{media_client, start_harness_with_requests};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_net::media_request_executor::MediaRequestLimits;

#[tokio::test]
async fn expired_redirect_admission_yields_to_body() {
    let origin = delivery_redirect_admission_timeout_origin::serve().await;
    let requests = media_client();
    requests.update_limits(MediaRequestLimits::try_new(2, 1).expect("request limits"));
    let occupied = requests
        .get(&origin.blocked_url, PreemptionAuthority::Transition)
        .expect("blocked request")
        .admit()
        .await
        .expect("occupy redirect authority");
    let mut options = DeliveryOptions::default();
    options.tuning.max_requests_per_authority = Some(NonZeroUsize::MIN);
    let harness = start_harness_with_requests("redirect-head-timeout", options, requests);
    let current = serve_visible_current().await;
    harness.handle.update_focus(focus_now(
        vec![current.item(), unsized_item("future", &origin.url)],
        0,
        0,
    ));
    current.assert_get_without_head().await;
    tokio::time::timeout(Duration::from_secs(30), origin.head_started)
        .await
        .expect("HEAD request start")
        .expect("origin start signal");

    let observed = tokio::time::timeout(Duration::from_secs(4), origin.requests)
        .await
        .expect("redirect timeout must yield to body")
        .expect("origin task");
    assert!(observed.head.starts_with(b"HEAD "));
    assert!(
        observed.body.starts_with(b"GET "),
        "second request was {}",
        String::from_utf8_lossy(&observed.body)
    );

    drop(occupied);
    harness.handle.clear().await.expect("clear delivery");
    std::fs::remove_dir_all(&harness.root).ok();
}
