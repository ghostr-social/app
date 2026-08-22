//! An in-flight HEAD contributes to the real manager's request-pressure evidence.

mod delivery_fixture;
mod raw_http;

use delivery_fixture::items::{focus_now, unsized_item};
use delivery_fixture::options::serial_long_retry_options;
use delivery_fixture::start_harness;
use ghostr_delivery::delivery_events::DeliveryHandle;
use raw_http::spawn_stalled_headers;
use std::time::Duration;

#[tokio::test]
async fn an_active_head_occupies_the_manager_request_budget() {
    let first = spawn_stalled_headers().await;
    let second = spawn_stalled_headers().await;
    let harness = start_harness("warp-active-probe-budget", serial_long_retry_options(1));
    harness
        .handle
        .update_focus(focus_now(vec![unsized_item("first", &first.url)], 0, 0));
    tokio::time::timeout(Duration::from_secs(2), first.request_started)
        .await
        .expect("HEAD starts in time")
        .expect("HEAD reaches its origin");
    let revision = harness.handle.latest_plan().expect("initial plan").revision;
    let before = harness.handle.evaluation_snapshot().budget;

    harness
        .handle
        .update_focus(focus_now(vec![unsized_item("second", &second.url)], 0, 0));
    wait_for_plan_after(&harness.handle, revision, "second").await;
    let after = harness.handle.evaluation_snapshot().budget;

    assert!(after.observations > before.observations);
    assert!(
        after.long_run_network_target_error_bps > before.long_run_network_target_error_bps,
        "a live HEAD must increase observed request pressure"
    );
    let mut second_started = second.request_started;
    assert!(
        tokio::time::timeout(Duration::from_millis(250), &mut second_started)
            .await
            .is_err(),
        "the occupied request budget must block the second HEAD"
    );

    first.requests.abort();
    let _ = first.requests.await;
    tokio::time::timeout(Duration::from_secs(2), &mut second_started)
        .await
        .expect("second HEAD starts after release")
        .expect("second HEAD reaches its origin");
    second.requests.abort();
    let _ = second.requests.await;
    harness.handle.clear().await.unwrap();
    std::fs::remove_dir_all(&harness.root).ok();
}

async fn wait_for_plan_after(handle: &DeliveryHandle, revision: u64, current: &str) {
    let notifier = handle.plan_notifier();
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let changed = notifier.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if handle.latest_plan().is_some_and(|plan| {
                plan.revision > revision
                    && plan
                        .current
                        .as_ref()
                        .is_some_and(|post| post.as_str() == current)
            }) {
                return;
            }
            changed.await;
        }
    })
    .await
    .expect("manager publishes the next planning pass");
}
