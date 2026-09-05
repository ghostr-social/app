//! A post whose every source failed is terminal: it stops being
//! retried by later replanning passes and stops being advertised as
//! servable, instead of being rescheduled forever.

mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, hits, media_body, serve_recording, serve_rejecting};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use tokio::time::Instant;

#[tokio::test]
async fn delivery_manager_stops_retrying_a_post_with_no_working_source() {
    let log = hit_log();
    let broken = serve_rejecting("broken", std::sync::Arc::clone(&log)).await;
    let live = serve_recording("live", media_body(), std::sync::Arc::clone(&log)).await;
    let harness = start_harness("ghostr-delivery-exhausted", DeliveryOptions::default());
    let doomed = sized_item("aa11", &broken, 16, 1_000);

    harness
        .handle
        .update_focus(focus_now(vec![doomed.clone()], 0, 0));
    wait_for("the doomed post to go terminal", || {
        attempts(&log) > 0
            && harness
                .posts
                .videos()
                .iter()
                .any(|video| video.id == "aa11" && !video.status.is_servable())
    })
    .await;
    tokio::time::sleep(Duration::from_millis(150)).await;
    let settled = attempts(&log);

    // Every later focus update replans the doomed post; none may retry
    // it. The fresh post proves those passes really ran.
    for _ in 0..3 {
        harness
            .handle
            .update_focus(focus_now(vec![doomed.clone()], 0, 0));
    }
    harness.handle.update_focus(focus_now(
        vec![doomed, sized_item("bb22", &live, 16, 1_000)],
        1,
        0,
    ));
    wait_for_ranges(&harness.store, "bb22", &[(0, 16)]).await;

    assert_eq!(
        attempts(&log),
        settled,
        "a terminal post must stay terminal"
    );
    std::fs::remove_dir_all(&harness.root).ok();
}

fn attempts(log: &delivery_fixture::media::HitLog) -> usize {
    hits(log)
        .iter()
        .filter(|hit| hit.starts_with("broken:"))
        .count()
}

async fn wait_for(what: &str, ready: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !ready() {
        assert!(Instant::now() < deadline, "timed out waiting for {what}");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
