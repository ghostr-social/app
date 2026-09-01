use super::delivery_fixture::concurrency_origin::{ActiveRequest, ControlledOrigin};
use super::delivery_fixture::DeliveryHarness;
use core::ops::Range;
use core::time::Duration;
use ghostr_delivery::delivery_events::DeliveryHandle;
mod decision_summary;
mod stream;
pub(super) use stream::next_request_while_streaming;
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
pub(super) async fn expect_no_request(origin: &mut ControlledOrigin, handle: &DeliveryHandle) {
    let request = tokio::time::timeout(Duration::from_millis(100), origin.next()).await;
    if let Ok(request) = request {
        panic!(
            "concurrency rose before enough evidence: range={:?}; decisions={:?}",
            request.range,
            decision_summary::summarize(&history(handle))
        );
    }
}
pub(super) async fn wait_for_parallel_demand_after(handle: &DeliveryHandle, after: u64) {
    tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            if latest_decision(&history(handle))
                .is_some_and(|(sequence, demanded)| sequence > after && demanded)
            {
                return;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("WARP reports positive parallel demand");
}

pub(super) fn decision_sequence(handle: &DeliveryHandle) -> u64 {
    latest_decision(&history(handle))
        .map(|decision| decision.0)
        .expect("post-admission WARP decision evidence")
}

fn latest_decision(history: &str) -> Option<(u64, bool)> {
    let value = serde_json::from_str::<serde_json::Value>(history).ok()?;
    value["decisions"]["records"]
        .as_array()
        .and_then(|records| records.last())
        .and_then(|record| {
            Some((
                record["sequence"].as_u64()?,
                record["warp_decision"]["additional_request_slot_demanded"].as_bool()?,
            ))
        })
}

fn history(handle: &DeliveryHandle) -> String {
    handle
        .decision_history_json()
        .expect("serializable decision evidence")
}
pub(super) async fn wait_for_bytes(harness: &DeliveryHarness, expected: u64) {
    tokio::time::timeout(Duration::from_secs(10), async {
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
