//! A failed body must not make an expired advisory HEAD eligible again.

mod delivery_fixture;
mod delivery_head_timeout_failed_body_retry_origin;

use core::num::NonZeroUsize;
use core::time::Duration;
use delivery_fixture::head_window::serve_visible_current;
use delivery_fixture::items::{focus_now, unsized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;

#[tokio::test]
async fn failed_body_retry_does_not_rearm_timed_out_head() {
    let origin = delivery_head_timeout_failed_body_retry_origin::serve().await;
    let mut options = DeliveryOptions::default();
    options.tuning.max_requests_per_authority = Some(NonZeroUsize::MIN);
    let harness = start_harness("head-timeout-failed-body", options);
    let current = serve_visible_current().await;
    harness.handle.update_focus(focus_now(
        vec![current.item(), unsized_item("future", &origin.url)],
        0,
        0,
    ));
    current.assert_get_without_head().await;

    let observed = tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            let methods = origin.methods();
            if methods.len() >= 3 {
                break methods;
            }
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await;
    assert_eq!(
        observed.unwrap_or_else(|_| origin.methods()),
        ["HEAD", "GET", "GET"]
    );

    harness.handle.clear().await.expect("clear delivery");
    std::fs::remove_dir_all(&harness.root).ok();
}
