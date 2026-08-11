mod delivery_fixture;

use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::media::{hit_log, media_body, serve_recording};
use delivery_fixture::options::serial_long_retry_options;
use delivery_fixture::partial_failure_origin::{serve, PartialFailureOrigin};
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_delivery::playback_demand::DemandSignal;
use ghostr_engine::{ByteRange, PostId};
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::time::Duration;
use tokio::time::Instant;

#[tokio::test]
async fn each_new_playback_gap_gets_one_immediate_retry() {
    let healthy = serve_recording("healthy", media_body(), hit_log()).await;
    let target = serve(16).await;
    let harness = start_harness("ghostr-demand-cooldown-retry", serial_long_retry_options(6));
    harness.handle.update_focus(focus_now(
        vec![
            sized_item("current", &healthy, 16, 1_000),
            sized_item("playing", target.url(), 16, 1_000),
            sized_item("barrier", &healthy, 16, 1_000),
        ],
        0,
        0,
    ));
    wait_for_ranges(&harness.store, "barrier", &[(0, 16)]).await;
    let first_end = wait_for_progress(&harness.store, 0).await;
    harness.handle.update_focus(focus_now(
        vec![
            sized_item("playing", target.url(), 16, 1_000),
            sized_item("current", &healthy, 16, 1_000),
            sized_item("barrier", &healthy, 16, 1_000),
        ],
        0,
        0,
    ));
    wait_for_attempts(&target, 2).await;
    let second_end = wait_for_progress(&harness.store, first_end).await;

    harness.demand.emit(signal(second_end));
    wait_for_attempts(&target, 3).await;
    let third_end = wait_for_progress(&harness.store, second_end).await;
    harness.demand.emit(signal(second_end));
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(
        target.attempts(),
        3,
        "duplicate gap bypassed retry pacing: {:?}",
        target.starts()
    );

    harness.demand.emit(signal(third_end));
    wait_for_attempts(&target, 4).await;
    std::fs::remove_dir_all(&harness.root).ok();
}

fn signal(start: u64) -> DemandSignal {
    DemandSignal {
        post: PostId::new("playing"),
        range: ByteRange::new(start, start + 4),
    }
}

async fn wait_for_attempts(origin: &PartialFailureOrigin, expected: usize) {
    let deadline = Instant::now() + Duration::from_millis(500);
    while origin.attempts() < expected {
        assert!(
            Instant::now() < deadline,
            "timed out waiting for attempt {expected}"
        );
        tokio::task::yield_now().await;
    }
}

async fn wait_for_progress(store: &PartialRangeStore, after: u64) -> u64 {
    let deadline = Instant::now() + Duration::from_millis(500);
    loop {
        let ranges = store.present_ranges("playing").await.expect("ranges");
        let end = ranges.first().map_or(0, |range| range.end);
        if end > after {
            return end;
        }
        assert!(Instant::now() < deadline, "stored bytes did not advance");
        tokio::task::yield_now().await;
    }
}
