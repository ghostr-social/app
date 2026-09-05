mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::concurrency_origin::{ActiveRequest, ControlledOrigin};
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::production_geometry_parallel_options;
use delivery_fixture::start_harness;

const TOTAL: u64 = 293_999;

#[tokio::test]
async fn cold_start_bounds_current_and_future_work_across_distinct_origins() {
    let mut current_origin = ControlledOrigin::serve(TOTAL).await;
    let mut next_origin = ControlledOrigin::serve(TOTAL).await;
    let mut third_origin = ControlledOrigin::serve(TOTAL).await;
    let harness = start_harness(
        "ghostr-parallel-reserve",
        production_geometry_parallel_options(),
    );
    harness.handle.update_focus(focus_now(
        vec![
            sized_item("current", &current_origin.url, TOTAL, 6_000),
            sized_item("next", &next_origin.url, TOTAL, 6_000),
            sized_item("third", &third_origin.url, TOTAL, 6_000),
        ],
        0,
        0,
    ));

    let current = next_request(&mut current_origin).await;
    let next = next_request(&mut next_origin).await;
    assert_eq!(current.range.start, 0);
    assert_eq!(next.range, 0..65_536);
    assert!(
        current.is_open() && next.is_open(),
        "both protected origins make progress"
    );
    assert!(
        tokio::time::timeout(Duration::from_millis(100), third_origin.next())
            .await
            .is_err(),
        "a third origin cannot bypass the global two-request limit"
    );
    assert!(
        current.send_byte().await && next.send_byte().await,
        "both bodies stay readable"
    );
    harness.handle.clear().await.expect("clear delivery");
    std::fs::remove_dir_all(&harness.root).ok();
}

async fn next_request(origin: &mut ControlledOrigin) -> ActiveRequest {
    tokio::time::timeout(Duration::from_secs(5), origin.next())
        .await
        .expect("protected request starts")
}
