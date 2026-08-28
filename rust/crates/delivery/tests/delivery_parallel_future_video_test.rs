mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::concurrency_origin::{ActiveRequest, ControlledOrigin};
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;

#[tokio::test]
async fn positive_warp_demand_starts_a_future_video_without_learning_delay() {
    let mut origin = ControlledOrigin::serve(128).await;
    let harness = start_harness("ghostr-parallel-future", DeliveryOptions::default());
    harness.handle.update_focus(focus_now(
        vec![item("current", &origin), item("next", &origin)],
        0,
        0,
    ));

    let current = next_request(&mut origin).await;
    let next = next_request(&mut origin).await;

    assert_eq!(current.path, "/current.mp4");
    assert_eq!(next.path, "/next.mp4");
    assert!(current.is_open() && next.is_open());
    assert!(current.send_byte().await && next.send_byte().await);
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}

fn item(
    id: &'static str,
    origin: &ControlledOrigin,
) -> ghostr_delivery::delivery_events::FocusItem {
    sized_item(id, &origin.url_for(id), 128, 128_000)
}

async fn next_request(origin: &mut ControlledOrigin) -> ActiveRequest {
    tokio::time::timeout(Duration::from_secs(2), origin.next())
        .await
        .expect("parallel video request in time")
}
