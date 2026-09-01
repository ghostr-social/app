mod delivery_fixture;
#[path = "delivery_focus_pre_body_cancellation_test/fixture.rs"]
mod focus_fixture;
#[path = "delivery_cancelled_prefix_refocus_test/helper.rs"]
mod helper;
#[path = "delivery_cancelled_prefix_refocus_test/plan.rs"]
mod plan;
#[path = "delivery_cancelled_prefix_refocus_test/origin.rs"]
mod replacement_origin;
#[path = "delivery_cancelled_prefix_refocus_test/roster.rs"]
mod trimmed_roster;

use delivery_fixture::decision::wait_for_history;
use delivery_fixture::demand;
use delivery_fixture::options::production_geometry_parallel_options;
use delivery_fixture::start_harness;
use focus_fixture::{generated_focus, roster, seed_prefix, wait_response_open};
use ghostr_delivery::delivery_events::FocusAdmission;
use ghostr_engine::adaptive::{ControlMode, DecisionOutcome};
use ghostr_engine::ByteRange;
use helper::{next_request, pending_transfer_sequence, seed_tail, wait_closed};

const TOTAL: u64 = 293_999;
const PREFIX: ByteRange = ByteRange {
    start: 0,
    end: 65_536,
};
const TAIL: ByteRange = ByteRange {
    start: 262_144,
    end: TOTAL,
};

#[tokio::test]
async fn refocus_restarts_a_cancelled_demanded_prefix_beside_a_completed_tail() {
    let mut origin = delivery_fixture::concurrency_origin::ControlledOrigin::serve(TOTAL).await;
    let items = roster(&origin);
    let harness = start_harness(
        "ghostr-cancelled-prefix-refocus",
        production_geometry_parallel_options(),
    );
    seed_prefix(&harness, &items).await;
    seed_tail(&harness, &items[6]).await;
    for (generation, current) in (1..=4).zip(0..=3) {
        focus_fixture::focus_and_wait(&harness, &items, current, generation).await;
    }

    let prefix = next_request(&mut origin, "initial prefix").await;
    assert_eq!(
        (&prefix.path, prefix.range.clone()),
        (&"/p6.mp4".into(), 0..65_536)
    );
    wait_response_open(&harness, "p6").await;
    let target_sequence = pending_transfer_sequence(&harness.handle);
    let demand = demand::blocked(&harness, "p6", PREFIX).await;

    for (generation, current) in [(5, 2), (6, 1), (7, 0)] {
        assert_eq!(
            harness
                .handle
                .update_focus(generated_focus(items.clone(), current, generation)),
            FocusAdmission::Accepted
        );
    }
    wait_closed(&prefix).await;
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

    let refocused = trimmed_roster::sequential_refocus(&harness, &items).await;
    assert_eq!(refocused.plan.mode, ControlMode::Emergency);
    plan::assert_prefix_allocation(&refocused);
    let replacement = replacement_origin::next_prefix(&mut origin).await;
    drop(replacement);
    drop(demand);
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}
