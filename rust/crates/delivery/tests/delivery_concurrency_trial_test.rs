mod delivery_fixture;

use core::ops::Range;
use core::time::Duration;
use delivery_fixture::concurrency_origin::{ActiveRequest, ControlledOrigin};
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::playback::{playing_at, wait_for_admissions};
use delivery_fixture::{start_harness, DeliveryHarness};
use ghostr_engine::EngineParams;

const WINDOWS_FOR_TRIAL: usize = 4;
const BYTES_PER_WINDOW: usize = 8;
const SAMPLE_WINDOW: Duration = Duration::from_millis(520);

#[tokio::test]
async fn positive_warp_demand_starts_one_parallel_disjoint_range() {
    let mut origin = ControlledOrigin::serve(128).await;
    let harness = start_harness("ghostr-concurrency-trial", options());
    harness.handle.update_focus(focus_now(
        vec![sized_item("current", &origin.url, 128, 128_000)],
        0,
        0,
    ));

    let first = next_request(&mut origin).await;
    expect_no_request(&mut origin).await;
    for window in 1..=WINDOWS_FOR_TRIAL {
        harness.handle.report_playback(playing_at(
            "current",
            Duration::from_secs(8),
            window as u64,
        ));
        wait_for_admissions(&harness.handle, window as u64).await;
        send_bytes(&first).await;
        wait_for_bytes(&harness, (window * BYTES_PER_WINDOW) as u64).await;
        tokio::time::sleep(SAMPLE_WINDOW).await;
        if window < WINDOWS_FOR_TRIAL {
            expect_no_request(&mut origin).await;
        }
    }

    let second = next_request(&mut origin).await;
    assert!(
        disjoint(first.range.clone(), second.range.clone()),
        "parallel ranges overlap: {:?} and {:?}",
        first.range,
        second.range
    );
    assert!(first.send_byte().await, "trial preserves the first range");
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}

async fn send_bytes(request: &ActiveRequest) {
    for _ in 0..BYTES_PER_WINDOW {
        assert!(request.send_byte().await, "first range remains active");
    }
}

async fn next_request(origin: &mut ControlledOrigin) -> ActiveRequest {
    tokio::time::timeout(Duration::from_secs(2), origin.next())
        .await
        .expect("range request in time")
}

async fn expect_no_request(origin: &mut ControlledOrigin) {
    let result = tokio::time::timeout(Duration::from_millis(100), origin.next()).await;
    assert!(result.is_err(), "concurrency rose before enough evidence");
}

async fn wait_for_bytes(harness: &DeliveryHarness, expected: u64) {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let ranges = harness
                .store
                .present_ranges("current")
                .await
                .expect("valid test fixture");
            let stored: u64 = ranges.iter().map(|range| range.end - range.start).sum();
            if stored >= expected {
                return;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("progress reaches the store");
}

fn disjoint(first: Range<u64>, second: Range<u64>) -> bool {
    first.end <= second.start || second.end <= first.start
}

fn options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams {
            chunk_bytes: 64,
            ..base_params()
        },
        ..DeliveryOptions::default()
    }
}
