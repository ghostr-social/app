use super::delivery_fixture::concurrency_origin::{ActiveRequest, ControlledOrigin};
use super::delivery_fixture::DeliveryHarness;
use core::ops::Range;
use core::time::Duration;
use ghostr_delivery::delivery_events::DeliveryHandle;
pub(super) async fn next_request(
    origin: &mut ControlledOrigin,
    handle: &DeliveryHandle,
    phase: &str,
) -> ActiveRequest {
    match tokio::time::timeout(Duration::from_secs(10), origin.next()).await {
        Ok(request) => request,
        Err(_) => panic!(
            "{phase} timed out; decisions={}",
            handle
                .decision_history_json()
                .expect("serializable diagnostic evidence")
        ),
    }
}
pub(super) async fn expect_no_request(origin: &mut ControlledOrigin) {
    let result = tokio::time::timeout(Duration::from_millis(100), origin.next()).await;
    assert!(result.is_err(), "concurrency rose before enough evidence");
}
pub(super) async fn wait_for_parallel_demand(handle: &DeliveryHandle) {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            let history = handle
                .decision_history_json()
                .expect("serializable decision evidence");
            if latest_parallel_demand(&history) {
                return;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("WARP reports positive parallel demand");
}

fn latest_parallel_demand(history: &str) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(history) else {
        return false;
    };
    value["decisions"]["records"]
        .as_array()
        .and_then(|records| records.last())
        .and_then(|record| record["warp_decision"]["additional_request_slot_demanded"].as_bool())
        .unwrap_or(false)
}
pub(super) async fn wait_for_bytes(harness: &DeliveryHarness, expected: u64) {
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

pub(super) fn disjoint(first: Range<u64>, second: Range<u64>) -> bool {
    first.end <= second.start || second.end <= first.start
}
