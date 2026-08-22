//! Follow-up work after a selected HEAD remains bound to selected WARP decisions.

mod delivery_fixture;
mod raw_http;

use delivery_fixture::items::{focus_now, unsized_item};
use delivery_fixture::options::DeliveryOptions;
use delivery_fixture::start_harness;
use ghostr_engine::adaptive::{DecisionOutcome, DecisionRecord, RecordedWarpCommand};
use raw_http::spawn_stalled_headers;

#[tokio::test]
async fn unresolved_window_binds_followup_work_to_selected_decisions() {
    let first = spawn_stalled_headers().await;
    let second = spawn_stalled_headers().await;
    let harness = start_harness("warp-selected-head", DeliveryOptions::default());

    harness.handle.update_focus(focus_now(
        vec![
            unsized_item("first", &first.url),
            unsized_item("second", &second.url),
        ],
        0,
        0,
    ));

    first
        .request_started
        .await
        .expect("selected HEAD must reach the first origin");
    second
        .request_started
        .await
        .expect("selected follow-up work must reach the second origin");
    let records = harness.handle.decision_history().records;
    assert_eq!(pending_heads(&records), 1);
    assert!(bound_pending_transfers(&records) >= 1);
    harness.handle.clear().await.unwrap();
    std::fs::remove_dir_all(&harness.root).ok();
}

fn pending_heads(records: &[DecisionRecord]) -> usize {
    records
        .iter()
        .filter(|record| record.eventual_outcome == DecisionOutcome::Pending)
        .filter(|record| {
            selected_is(record, |command| {
                matches!(command, RecordedWarpCommand::ProbeHead { .. })
            })
        })
        .count()
}

fn bound_pending_transfers(records: &[DecisionRecord]) -> usize {
    records
        .iter()
        .filter(|record| record.eventual_outcome == DecisionOutcome::Pending)
        .filter(|record| record.executed_request.is_some())
        .filter(|record| {
            selected_is(record, |command| {
                matches!(command, RecordedWarpCommand::Transfer { .. })
            })
        })
        .count()
}

fn selected_is(record: &DecisionRecord, predicate: impl Fn(&RecordedWarpCommand) -> bool) -> bool {
    record
        .warp_decision
        .as_ref()
        .and_then(|warp| warp.selected.as_ref())
        .is_some_and(|action| predicate(&action.command))
}
