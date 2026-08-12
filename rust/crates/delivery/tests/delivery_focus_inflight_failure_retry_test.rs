mod delivery_fixture;

use delivery_fixture::gated_failure::serve;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, media_body, serve_recording};
use delivery_fixture::options::serial_long_retry_options;
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_cache_first;
use std::time::Duration;

#[tokio::test]
async fn focus_before_failure_grants_exactly_one_immediate_retry() {
    let healthy = serve_recording("healthy", media_body(), hit_log()).await;
    let mut failure = serve().await;
    let harness = start_harness(
        "ghostr-focus-inflight-failure",
        serial_long_retry_options(4),
    );
    let current = sized_item("current", &healthy, 16, 1_000);
    let target = sized_item("target", failure.url(), 16, 1_000);
    harness
        .handle
        .update_focus(focus_now(vec![current.clone(), target.clone()], 0, 0));
    failure.wait_started().await;

    harness
        .handle
        .update_focus(focus_now(vec![target, current], 0, 0));
    wait_cache_first(&harness.cache, "target").await;
    failure.release();

    failure.wait_for_attempts(2).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(failure.attempts(), 2, "focus credit was reusable");
    std::fs::remove_dir_all(&harness.root).ok();
}
