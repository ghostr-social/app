mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::gated_failure::serve;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, media_body, serve_recording};
use delivery_fixture::options::serial_long_retry_options;
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_cache_first;

#[tokio::test]
async fn focus_credit_expires_when_the_viewer_leaves_before_failure() {
    let healthy = serve_recording("healthy", media_body(), hit_log()).await;
    let mut failure = serve().await;
    let harness = start_harness("ghostr-stale-focus-credit", serial_long_retry_options(4));
    let current = sized_item("current", &healthy, 16, 1_000);
    let target = sized_item("target", failure.url(), 16, 1_000);
    harness
        .handle
        .update_focus(focus_now(vec![current.clone(), target.clone()], 0, 0));
    failure.wait_started().await;

    harness
        .handle
        .update_focus(focus_now(vec![target.clone(), current.clone()], 0, 0));
    wait_cache_first(&harness.cache, "target").await;
    harness
        .handle
        .update_focus(focus_now(vec![current, target], 0, 0));
    wait_cache_first(&harness.cache, "current").await;
    failure.release();

    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(failure.attempts(), 1, "departed focus kept retry credit");
    std::fs::remove_dir_all(&harness.root).ok();
}
