mod delivery_fixture;
#[path = "delivery_focus_pre_body_cancellation_test/fixture.rs"]
mod fixture;

use core::time::Duration;
use delivery_fixture::options::production_geometry_parallel_options;
use delivery_fixture::plan::wait_for_plan;
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use fixture::{generated_focus, roster, seed_prefix, wait_response_open};
use ghostr_delivery::delivery_events::FocusAdmission;

const TOTAL: u64 = 32;

#[tokio::test]
async fn reverse_focus_preserves_a_future_request_after_its_first_body_byte() {
    let mut held = delivery_fixture::concurrency_origin::ControlledOrigin::serve(TOTAL).await;
    let items = roster(&held);
    let harness = start_harness(
        "ghostr-focus-partial-body-retain",
        production_geometry_parallel_options(),
    );
    seed_prefix(&harness, &items).await;
    for (generation, current) in (1..=4).zip(0..=3) {
        fixture::focus_and_wait(&harness, &items, current, generation).await;
    }
    let request = tokio::time::timeout(Duration::from_secs(10), held.next())
        .await
        .expect("future request starts");
    assert_eq!(request.path, "/p6.mp4");
    wait_response_open(&harness, "p6").await;
    assert!(request.send_byte().await, "first body byte is accepted");
    wait_for_ranges(&harness.store, "p6", &[(0, 1)]).await;
    let after = harness.handle.latest_plan().expect("focused plan").revision;

    for (generation, current) in [(5, 2), (6, 1), (7, 0)] {
        assert_eq!(
            harness
                .handle
                .update_focus(generated_focus(items.clone(), current, generation)),
            FocusAdmission::Accepted
        );
    }
    wait_for_plan(&harness.handle, after, |plan| {
        plan.focus_generation == Some(7)
    })
    .await;

    assert!(
        request.is_open(),
        "useful partial-body work remains eligible"
    );
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}
