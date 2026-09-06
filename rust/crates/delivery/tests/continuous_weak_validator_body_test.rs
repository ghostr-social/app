//! A weak-validator object larger than the rate burst must still reach playback.
mod continuous_body_fixture;
mod delivery_fixture;

use continuous_body_fixture::{serve, wait_at, TOTAL};
use delivery_fixture::items::{focus_now, sized_item};

#[tokio::test]
async fn weak_validator_body_larger_than_burst_exposes_a_continuous_prefix() {
    let origin = serve(false).await;
    let harness = continuous_body_fixture::harness("continuous-weak-validator");
    let item = sized_item("current", &origin.url, TOTAL, 60_000);
    harness.handle.update_focus(focus_now(vec![item], 0, 0));
    let prefix = tokio::time::timeout(
        core::time::Duration::from_secs(10),
        wait_at(&harness.store, 0),
    )
    .await;
    harness.handle.update_focus(focus_now(Vec::new(), 0, 0));
    assert!(
        prefix.is_ok(),
        "no readable prefix; whole responses started: {}",
        origin
            .whole_requests
            .load(core::sync::atomic::Ordering::Relaxed)
    );
    assert_eq!(
        origin
            .whole_requests
            .load(core::sync::atomic::Ordering::Relaxed),
        1
    );
}
