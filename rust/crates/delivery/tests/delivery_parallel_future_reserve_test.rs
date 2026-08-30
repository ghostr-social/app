mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::concurrency_origin::{ActiveRequest, ControlledOrigin};
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::production_geometry_parallel_options;
use delivery_fixture::start_harness;
use std::collections::HashMap;

const PREFIX_BYTES: u64 = 65_536;
const REQUEST_DEADLINE: Duration = Duration::from_secs(5);

#[tokio::test]
async fn cold_start_holds_two_future_prefixes_beside_current() {
    let mut origin = ControlledOrigin::serve(293_999).await;
    let harness = start_harness(
        "ghostr-parallel-future-reserve",
        production_geometry_parallel_options(),
    );
    harness.handle.update_focus(focus_now(
        vec![
            item("current", &origin),
            item("next", &origin),
            item("third", &origin),
        ],
        0,
        0,
    ));

    let mut active = requests(&mut origin, 3).await;
    let current = active.remove("/current.mp4").expect("current prefix");
    let next = active.remove("/next.mp4").expect("next prefix");
    let third = active.remove("/third.mp4").expect("second future prefix");
    assert_eq!(current.range.start, 0);
    assert_eq!(next.range, 0..PREFIX_BYTES);
    assert_eq!(third.range, 0..PREFIX_BYTES);
    assert!(current.is_open() && next.is_open() && third.is_open());
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}

fn item(
    id: &'static str,
    origin: &ControlledOrigin,
) -> ghostr_delivery::delivery_events::FocusItem {
    sized_item(id, &origin.url_for(id), 293_999, 6_000)
}

async fn requests(origin: &mut ControlledOrigin, count: usize) -> HashMap<String, ActiveRequest> {
    let mut requests = HashMap::new();
    for _ in 0..count {
        let request = tokio::time::timeout(REQUEST_DEADLINE, origin.next())
            .await
            .unwrap_or_else(|_| {
                panic!("parallel requests in time; observed={:?}", requests.keys())
            });
        requests.insert(request.path.clone(), request);
    }
    requests
}
