//! A weak-validator object larger than the rate burst must still reach playback.
mod delivery_fixture;
mod continuous_body_fixture;

use continuous_body_fixture::{serve, wait_at, TOTAL};
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::production_geometry_parallel_options;

#[tokio::test]
async fn weak_validator_body_larger_than_burst_exposes_a_continuous_prefix() {
    let origin = serve().await;
    let harness = delivery_fixture::start_harness(
        "continuous-weak-validator", production_geometry_parallel_options(),
    );
    let item = sized_item("current", &origin.url, TOTAL, 60_000);
    harness.handle.update_focus(focus_now(vec![item], 0, 0));
    let prefix = tokio::time::timeout(
        core::time::Duration::from_secs(10), wait_at(&harness.store, 0),
    ).await;
    harness.handle.update_focus(focus_now(Vec::new(), 0, 0));
    assert!(prefix.is_ok(), "a usable origin must not be parked behind an impossible full-file rate reservation");
    assert_eq!(origin.whole_requests.load(std::sync::atomic::Ordering::Relaxed), 1);
}
