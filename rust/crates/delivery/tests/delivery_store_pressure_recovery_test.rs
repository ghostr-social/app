mod delivery_fixture;

use core::time::Duration;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::pressure_origin::serve;
use delivery_fixture::pressure_store::movable_store;
use delivery_fixture::start_harness_with_store;

#[tokio::test]
async fn admitted_body_recovers_when_capacity_disappears_before_its_write() {
    let mut origin = serve().await;
    let (store, space, root) = movable_store("ghostr-pressure-race");
    let mut options = DeliveryOptions::default();
    options.params.balanced_concurrency = 1;
    options.tuning.store_pressure_pause = Duration::from_millis(250);
    let harness = start_harness_with_store(store, root.clone(), options);
    harness.handle.update_focus(focus_now(
        vec![sized_item("current", &origin.url, 16, 1_000)],
        0,
        0,
    ));
    let refused_range = origin.wait_for_body().await;

    space.set(0);
    origin.release();
    wait_for_refusal(&harness).await;
    space.set(32);
    wait_for_complete(&harness).await;

    let requests = origin.requests();
    assert_eq!(requests.first(), Some(&refused_range));
    assert_eq!(
        requests.last(),
        Some(&(0, 15)),
        "same source completes through whole fallback"
    );
    assert_eq!(
        requests.len(),
        2,
        "one refused attempt and one successful fallback"
    );
    assert_eq!(harness.store.refusals(), 1);
    harness.handle.clear().await.expect("valid test fixture");
    drop(harness);
    std::fs::remove_dir_all(root).ok();
}

async fn wait_for_refusal(harness: &delivery_fixture::DeliveryHarness) {
    tokio::time::timeout(Duration::from_secs(1), async {
        while harness.store.refusals() == 0 {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    })
    .await
    .expect("admitted write reaches local pressure");
}

async fn wait_for_complete(harness: &delivery_fixture::DeliveryHarness) {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            let ranges = harness
                .store
                .present_ranges("current")
                .await
                .expect("valid test fixture");
            if ranges.len() == 1 && ranges[0] == (0..16) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap_or_else(|_| {
        panic!(
            "capacity recheck resumes source: {:#?}",
            harness.handle.latest_plan()
        )
    });
}
