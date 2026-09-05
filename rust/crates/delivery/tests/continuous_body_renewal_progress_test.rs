//! Reading beyond the initial window keeps the same HTTP response alive.
mod delivery_fixture;
mod continuous_body_fixture;

use continuous_body_fixture::{serve, wait_at, TOTAL};
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::production_geometry_parallel_options;

#[tokio::test]
async fn continuous_body_renews_beyond_initial_burst_without_restarting_the_request() {
    let origin = serve().await;
    let harness = delivery_fixture::start_harness(
        "continuous-renewals", production_geometry_parallel_options(),
    );
    let item = sized_item("current", &origin.url, TOTAL, 60_000);
    harness.handle.update_focus(focus_now(vec![item], 0, 0));
    let progress = tokio::time::timeout(
        core::time::Duration::from_secs(20), wait_at(&harness.store, 4 * 1024 * 1024),
    ).await;
    harness.handle.update_focus(focus_now(Vec::new(), 0, 0));
    assert!(progress.is_ok(), "continuous response must renew enough windows to exceed its initial burst");
    assert_eq!(origin.whole_requests.load(std::sync::atomic::Ordering::Relaxed), 1);
}
