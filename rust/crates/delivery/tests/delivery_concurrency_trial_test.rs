mod delivery_fixture;
#[path = "delivery_concurrency_trial_test/support.rs"]
mod support;

use core::num::NonZeroUsize;
use core::time::Duration;
use delivery_fixture::concurrency_origin::{ActiveRequest, ControlledOrigin};
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::playback::{playing_at, wait_for_admissions};
use delivery_fixture::stats::seed_overall_throughput;
use delivery_fixture::{start_harness_at, temp_directory, DeliveryHarness};
use ghostr_engine::EngineParams;
use support::{
    decision_sequence, disjoint, expect_no_request, next_request, next_request_while_streaming,
    wait_for_bytes, wait_for_parallel_demand_after,
};

const WARMUP_BYTES: usize = 8;
const SAMPLE_WINDOW: Duration = Duration::from_millis(520);
const TOTAL_BYTES: u64 = 9 * 1_024 * 1_024;

#[tokio::test]
async fn positive_warp_demand_starts_one_parallel_disjoint_range() {
    let mut origin = ControlledOrigin::serve(TOTAL_BYTES).await;
    let root = temp_directory("ghostr-concurrency-trial");
    seed_overall_throughput(&root, 1_048_576);
    let harness = start_harness_at(root, options());
    harness.handle.update_focus(focus_now(
        vec![sized_item(
            "current",
            &origin.url_for("current"),
            TOTAL_BYTES,
            128_000,
        )],
        0,
        0,
    ));

    let first = next_request(&mut origin, &harness.handle, "first request").await;
    assert_eq!(first.path, "/current.mp4");
    assert!(
        first.range.end < TOTAL_BYTES,
        "fixture must leave useful work"
    );
    expect_no_request(&mut origin, &harness.handle).await;
    assert!(first.send_byte().await, "first response opens");
    wait_for_bytes(&harness, 1).await;
    harness
        .handle
        .report_playback(playing_at("current", Duration::from_secs(4), 1));
    wait_for_admissions(&harness.handle, 1).await;
    let demand_fence = decision_sequence(&harness.handle);
    send_bytes(&first).await;
    wait_for_bytes(&harness, WARMUP_BYTES as u64 + 1).await;
    tokio::time::sleep(SAMPLE_WINDOW).await;
    wait_for_parallel_demand_after(&harness.handle, demand_fence).await;
    expect_no_request(&mut origin, &harness.handle).await;
    let second = next_request_while_streaming(&mut origin, &first, &harness.handle).await;
    finish_trial(&harness, first, second).await;
}

async fn finish_trial(harness: &DeliveryHarness, first: ActiveRequest, second: ActiveRequest) {
    assert_eq!(
        second.path, "/current.mp4",
        "parallel trial remains on the active item"
    );
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
    for _ in 0..WARMUP_BYTES {
        assert!(request.send_byte().await, "first range remains active");
    }
}

fn options() -> DeliveryOptions {
    let mut options = DeliveryOptions {
        params: EngineParams {
            chunk_bytes: 1_024 * 1_024,
            balanced_concurrency: 2,
            ..base_params()
        },
        ..DeliveryOptions::default()
    };
    options.tuning.max_requests_per_authority = NonZeroUsize::new(2);
    options
}
