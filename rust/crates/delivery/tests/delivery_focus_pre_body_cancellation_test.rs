mod delivery_fixture;
#[path = "delivery_focus_pre_body_cancellation_test/fixture.rs"]
mod fixture;
use core::time::Duration;
use delivery_fixture::decision::wait_for_history;
use delivery_fixture::evidence::DeliveryEvidence as _;
use delivery_fixture::options::production_geometry_parallel_options;
use delivery_fixture::start_harness;
use fixture::{generated_focus, roster, seed_prefix, wait_response_open};
use ghostr_delivery::delivery_events::{DeliveryHandle, FocusAdmission};
use ghostr_engine::adaptive::DecisionOutcome;
const TOTAL: u64 = 32;

#[tokio::test]
async fn reverse_focus_cancels_a_future_response_before_its_first_body_byte() {
    let mut held = delivery_fixture::concurrency_origin::ControlledOrigin::serve(TOTAL).await;
    let items = roster(&held);
    let harness = start_harness(
        "ghostr-focus-pre-body-cancel",
        production_geometry_parallel_options(),
    );
    seed_prefix(&harness, &items).await;
    for (generation, current) in (1..=4).zip(0..=3) {
        fixture::focus_and_wait(&harness, &items, current, generation).await;
    }
    let request = tokio::time::timeout(Duration::from_secs(10), held.next())
        .await
        .expect("future request starts");
    wait_response_open(&harness, "p6").await;
    assert_eq!(request.path, "/p6.mp4");
    assert!(
        harness
            .store
            .present_ranges("p6")
            .await
            .expect("ranges")
            .is_empty(),
        "response must still be waiting for its first body byte"
    );
    let target_sequence = pending_transfer_sequence(&harness.handle);
    for (generation, current) in [(5, 2), (6, 1), (7, 0)] {
        assert_eq!(
            harness
                .handle
                .update_focus(generated_focus(items.clone(), current, generation)),
            FocusAdmission::Accepted
        );
    }

    tokio::time::timeout(Duration::from_secs(1), async {
        while request.is_open() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("receding zero-byte request is cancelled");
    wait_for_history(&harness.handle, |history| {
        history.records.iter().any(|record| {
            record.sequence == target_sequence
                && matches!(
                    record.eventual_outcome,
                    DecisionOutcome::Cancelled { bytes: 0, .. }
                )
        })
    })
    .await;
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}

fn pending_transfer_sequence(handle: &DeliveryHandle) -> u64 {
    let history = handle.decision_history();
    let mut pending = history.records.iter().filter(|record| {
        record.chosen_action_id.is_some() && record.eventual_outcome == DecisionOutcome::Pending
    });
    let sequence = pending.next().expect("bound future transfer").sequence;
    assert!(pending.next().is_none(), "one bound future transfer");
    sequence
}
