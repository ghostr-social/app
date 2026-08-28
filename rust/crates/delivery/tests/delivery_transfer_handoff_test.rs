#[path = "delivery_transfer_handoff_test/assertions.rs"]
mod assertions;
mod delivery_fixture;

use assertions::{assert_decisions, is_handoff};
use core::time::Duration;
use delivery_fixture::concurrency_origin::{ActiveRequest, ControlledOrigin};
use delivery_fixture::decision::wait_for_history;
use delivery_fixture::evidence::DeliveryEvidence as _;
use delivery_fixture::items::{focus_now, sized_item};
use delivery_fixture::options::production_geometry_parallel_options;
use delivery_fixture::start_harness;
use ghostr_delivery::delivery_events::DeliveryHandle;

const TOTAL: u64 = 285_652;

#[tokio::test]
async fn detached_transfer_handoff_is_deferred_until_capacity_frees() {
    let mut old = ControlledOrigin::serve(TOTAL).await;
    let mut replacement = ControlledOrigin::serve(TOTAL).await;
    let mut options = production_geometry_parallel_options();
    options.params.aggressive_concurrency = 2;
    let harness = start_harness("ghostr-transfer-handoff", options);
    harness.handle.update_focus(focus_now(
        vec![sized_item("old", &old.url, TOTAL, 4_000)],
        0,
        0,
    ));
    let old_request = next_request("old", &mut old, &harness.handle).await;
    let cutoff = latest_sequence(&harness.handle);

    harness.handle.update_focus(focus_now(
        vec![sized_item("replacement", &replacement.url, TOTAL, 4_000)],
        0,
        0,
    ));
    wait_cancelled(&old_request).await;
    let replacement_request = next_request("replacement", &mut replacement, &harness.handle).await;
    assert_eq!(replacement_request.range.start, 0);
    expect_no_request(&mut replacement).await;
    wait_for_history(&harness.handle, |history| {
        history
            .records
            .iter()
            .any(|record| is_handoff(record, cutoff))
    })
    .await;

    assert_decisions(&harness.handle, cutoff);
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}

fn latest_sequence(handle: &DeliveryHandle) -> u64 {
    handle
        .decision_history()
        .records
        .last()
        .map_or(0, |record| record.sequence)
}

async fn next_request(
    label: &str,
    origin: &mut ControlledOrigin,
    handle: &DeliveryHandle,
) -> ActiveRequest {
    tokio::time::timeout(Duration::from_secs(2), origin.next())
        .await
        .unwrap_or_else(|_| {
            panic!(
                "{label} range request starts; plan={:#?}; decisions={:#?}",
                handle.latest_plan(),
                handle.decision_history()
            )
        })
}

async fn expect_no_request(origin: &mut ControlledOrigin) {
    assert!(
        tokio::time::timeout(Duration::from_millis(100), origin.next())
            .await
            .is_err()
    );
}

async fn wait_cancelled(request: &ActiveRequest) {
    tokio::time::timeout(Duration::from_secs(2), async {
        while request.is_open() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("detached request is cancelled");
}
