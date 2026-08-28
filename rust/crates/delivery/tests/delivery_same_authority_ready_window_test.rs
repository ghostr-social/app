mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::concurrency_origin::{ActiveRequest, ControlledOrigin};
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::production_geometry_parallel_options;
use delivery_fixture::playback::{playing, wait_for_admission};
use delivery_fixture::start_harness;
use ghostr_delivery::delivery_events::{DeliveryHandle, FocusItem};
use ghostr_engine::{adaptive::AllocationReason, ByteRange};
use std::collections::HashSet;

const FUTURE_BYTES: u64 = 285_652;
const PREFIX_BYTES: u64 = 65_536;

#[tokio::test]
async fn reserve_starts_two_future_videos_on_one_authority() {
    let mut origin = ControlledOrigin::serve(FUTURE_BYTES).await;
    let harness = start_harness(
        "ghostr-same-authority-ready",
        production_geometry_parallel_options(),
    );
    let current = prepare_current(&mut origin, &harness).await;
    harness.handle.update_focus(focus_now(
        vec![
            item("current", &origin),
            item("next", &origin),
            item("third", &origin),
            item("fourth", &origin),
        ],
        0,
        0,
    ));
    let first = next_request(&mut origin).await;
    assert_parallel_reserve_is_planned(&harness.handle);
    let second = next_request(&mut origin).await;

    assert_eq!(
        HashSet::from([first.path.as_str(), second.path.as_str()]),
        HashSet::from(["/next.mp4", "/third.mp4"]),
    );
    assert_eq!(first.range, 0..PREFIX_BYTES);
    assert_eq!(second.range, 0..PREFIX_BYTES);
    assert!(current.is_open(), "current continuation remains protected");
    assert!(first.send_byte().await);
    assert!(second.send_byte().await);
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}

async fn prepare_current(
    origin: &mut ControlledOrigin,
    harness: &delivery_fixture::DeliveryHarness,
) -> ActiveRequest {
    harness
        .handle
        .update_focus(focus_now(vec![item("current", origin)], 0, 0));
    let current = next_request(origin).await;
    assert_eq!(current.path, "/current.mp4");
    assert!(!current.range.is_empty());
    harness
        .handle
        .report_playback(playing("current", Duration::from_secs(20)));
    wait_for_admission(&harness.handle).await;
    current
}

fn assert_parallel_reserve_is_planned(handle: &DeliveryHandle) {
    let plan = handle.latest_plan().expect("published reserve plan").plan;
    let future: HashSet<_> = plan
        .allocations
        .iter()
        .filter(|allocation| matches!(allocation.post.as_str(), "next" | "third"))
        .inspect(|allocation| {
            assert_eq!(allocation.reason, AllocationReason::MediaBootstrap);
            assert_eq!(
                allocation.request.requested_bytes(),
                ByteRange::new(0, PREFIX_BYTES)
            );
        })
        .map(|allocation| allocation.post.as_str())
        .collect();
    assert!(plan.ready_reserve.target >= 2, "{plan:#?}");
    assert_eq!(future, HashSet::from(["next", "third"]), "{plan:#?}");
}

fn item(id: &'static str, origin: &ControlledOrigin) -> FocusItem {
    sized_item(id, &origin.url_for(id), FUTURE_BYTES, 4_000)
}

async fn next_request(origin: &mut ControlledOrigin) -> ActiveRequest {
    tokio::time::timeout(Duration::from_secs(2), origin.next())
        .await
        .expect("future video begins within the ready-window deadline")
}
