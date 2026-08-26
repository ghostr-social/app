mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::cooling_plan_origin::CoolingPlanOrigin;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::serial_long_retry_options;
use delivery_fixture::start_harness;
use ghostr_delivery::delivery_events::{DeliveryFocus, FocusAdmission};

#[tokio::test]
async fn cooling_protected_post_does_not_restart_useful_protected_io() {
    let origin = CoolingPlanOrigin::serve().await;
    let harness = start_harness("ghostr-cooling-plan-stability", options());
    let focus = window(&origin);
    assert_eq!(harness.handle.update_focus(focus), FocusAdmission::Accepted);
    tokio::time::timeout(Duration::from_secs(1), origin.wait_useful())
        .await
        .expect("useful protected transfer did not start");
    let failures = origin.failures();
    assert!(failures > 0, "fixture post did not fail");

    harness.handle.update_focus(window(&origin));
    let restarted = tokio::time::timeout(Duration::from_millis(150), origin.wait_useful())
        .await
        .is_ok();
    assert!(
        !restarted,
        "useful transfer was canceled and requested again"
    );
    assert_eq!(origin.useful_requests(), 1, "origin saw duplicate IO");

    origin.release();
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}

fn window(origin: &CoolingPlanOrigin) -> DeliveryFocus {
    focus_now(
        vec![
            sized_item("cooling", &origin.url("cooling"), 64, 1_000),
            sized_item("useful", &origin.url("useful"), 64, 1_000),
        ],
        0,
        0,
    )
}

fn options() -> delivery_fixture::options::DeliveryOptions {
    serial_long_retry_options(4)
}
