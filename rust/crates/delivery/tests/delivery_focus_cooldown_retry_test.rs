mod delivery_fixture;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, media_body, serve_recording};
use delivery_fixture::options::serial_long_retry_options;
use delivery_fixture::start_harness;
use delivery_fixture::transient_origin::{count, serve, Attempts};
use std::time::Duration;

#[tokio::test]
async fn newly_focused_post_retries_without_reviving_other_retry_state() {
    let healthy = serve_recording("healthy", media_body(), hit_log()).await;
    let (target, target_attempts) = serve().await;
    let (unrelated, unrelated_attempts) = serve().await;
    let harness = start_harness("ghostr-focus-cooldown-retry", serial_long_retry_options(2));

    harness
        .handle
        .update_focus(window(&healthy, &target, &unrelated));
    wait_for_attempts(&target_attempts, 1).await;
    wait_for_attempts(&unrelated_attempts, 1).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    harness
        .handle
        .update_focus(focused(&healthy, &target, &unrelated));

    wait_for_attempts(&target_attempts, 2).await;
    assert_eq!(
        count(&unrelated_attempts),
        1,
        "unfocused cooldown was cleared"
    );
    harness
        .handle
        .update_focus(focused(&healthy, &target, &unrelated));
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(count(&target_attempts), 2, "retired source was revived");
    std::fs::remove_dir_all(&harness.root).ok();
}

fn window(
    healthy: &str,
    target: &str,
    unrelated: &str,
) -> ghostr_delivery::delivery_events::DeliveryFocus {
    focus_now(
        vec![
            sized_item("current", healthy, 16, 1_000),
            sized_item("target", target, 16, 1_000),
            sized_item("unrelated", unrelated, 16, 1_000),
            sized_item("barrier", healthy, 16, 1_000),
        ],
        0,
        0,
    )
}

fn focused(
    healthy: &str,
    target: &str,
    unrelated: &str,
) -> ghostr_delivery::delivery_events::DeliveryFocus {
    let items = window(healthy, target, unrelated).items;
    focus_now(items, 1, 0)
}

async fn wait_for_attempts(attempts: &Attempts, expected: usize) {
    tokio::time::timeout(Duration::from_millis(500), async {
        while count(attempts) < expected {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("timed out waiting for attempt {expected}"));
}
