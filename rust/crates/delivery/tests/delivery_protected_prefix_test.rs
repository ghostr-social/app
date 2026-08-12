//! Origin IO stays inside the startup-protected focus prefix.

mod delivery_fixture;

use delivery_fixture::concurrency_origin::{ActiveRequest, ControlledOrigin};
use delivery_fixture::items::{focus_now, seed_range, sized_item};
use delivery_fixture::media::{hit_log, serve_recording};
use delivery_fixture::options::{base_params, DeliveryOptions};
use delivery_fixture::start_harness;
use delivery_fixture::wait::wait_for_ranges;
use ghostr_delivery::delivery_events::FocusItem;
use ghostr_engine::EngineParams;
use std::time::Duration;

const TOTAL: u64 = 1;
const IDS: [&str; 7] = ["v0", "v1", "v2", "v3", "v4", "v5", "v6"];

#[tokio::test]
async fn far_prefetch_waits_for_locality_and_then_starts_exactly_once() {
    let shared = serve_recording("shared", vec![1], hit_log()).await;
    let mut v5 = ControlledOrigin::serve(TOTAL).await;
    let mut v6 = ControlledOrigin::serve(TOTAL).await;
    let harness = start_harness("ghostr-protected-prefix", options());
    let items = items(&shared, &v5.url, &v6.url);
    for item in &items[1..=4] {
        seed_range(&harness.store, item, 0, &[1]).await;
    }

    harness.handle.update_focus(focus_now(items.clone(), 1, 0));
    assert_far_idle(&mut v6, &harness).await;
    expect_no_request(&mut v5).await;

    harness.handle.update_focus(focus_now(items.clone(), 2, 0));
    let v5_request = next_request(&mut v5).await;
    assert_far_idle(&mut v6, &harness).await;
    complete(v5_request).await;
    wait_for_ranges(&harness.store, "v5", &[(0, TOTAL)]).await;

    harness.handle.update_focus(focus_now(items, 3, 0));
    let v6_request = next_request(&mut v6).await;
    complete(v6_request).await;
    wait_for_ranges(&harness.store, "v6", &[(0, TOTAL)]).await;
    expect_no_request(&mut v6).await;

    harness.handle.clear().await.unwrap();
    std::fs::remove_dir_all(&harness.root).ok();
}

fn items(shared: &str, v5: &str, v6: &str) -> Vec<FocusItem> {
    IDS.iter()
        .enumerate()
        .map(|(index, id)| {
            let url = match index {
                5 => v5,
                6 => v6,
                _ => shared,
            };
            sized_item(id, url, TOTAL, 1_000)
        })
        .collect()
}

async fn next_request(origin: &mut ControlledOrigin) -> ActiveRequest {
    tokio::time::timeout(Duration::from_secs(2), origin.next())
        .await
        .expect("local origin request starts")
}

async fn expect_no_request(origin: &mut ControlledOrigin) {
    let result = tokio::time::timeout(Duration::from_millis(100), origin.next()).await;
    assert!(result.is_err(), "far origin request started");
}

async fn assert_far_idle(
    origin: &mut ControlledOrigin,
    harness: &delivery_fixture::DeliveryHarness,
) {
    expect_no_request(origin).await;
    let ranges = harness.store.present_ranges("v6").await.unwrap();
    assert!(ranges.is_empty(), "far origin body reached the store");
}

async fn complete(request: ActiveRequest) {
    assert!(request.send_byte().await, "origin body remains accepted");
    drop(request);
}

fn options() -> DeliveryOptions {
    DeliveryOptions {
        params: EngineParams {
            chunk_bytes: TOTAL,
            head_cap_bytes: TOTAL,
            balanced_concurrency: 2,
            ..base_params()
        },
        ..DeliveryOptions::default()
    }
}
