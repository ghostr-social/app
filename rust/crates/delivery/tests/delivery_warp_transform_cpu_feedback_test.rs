#![cfg(unix)]

mod delivery_fixture;
mod focus_wait_fixture;
mod priced_transform_delivery_fixture;
mod priced_transform_fixture;
mod transform_delivery_fixture;

use delivery_fixture::decision::wait_for_history;
use ghostr_delivery::delivery_events::DecisionHistorySnapshot;
use ghostr_engine::adaptive::{DecisionOutcome, DecisionRecord, RecordedWarpCommand};
use priced_transform_fixture::PricedRemux;
use std::sync::Arc;

#[tokio::test]
async fn measured_transform_changes_later_cpu_price_and_records_resources() {
    let harness =
        priced_transform_delivery_fixture::start("warp-transform-cpu-price", Arc::new(PricedRemux))
            .await;
    wait_for_history(&harness.handle, evidence_ready).await;

    let history = harness.handle.decision_history();
    let terminal = history
        .records
        .iter()
        .find(|record| transform(record))
        .unwrap();
    let actual = terminal
        .actual_resources
        .expect("actual Transform resources");
    assert!(actual.cpu_ms > 450 && actual.cpu_ms <= 500);
    assert_eq!(
        actual.storage_bytes,
        priced_transform_delivery_fixture::INPUT.len() as u64
    );
    let later = history
        .records
        .iter()
        .find(|record| later_cpu_price(record, terminal.sequence))
        .unwrap();
    assert!(later.warp_decision.as_ref().unwrap().prices.cpu_micros > 0);
    harness.handle.clear().await.unwrap();
    std::fs::remove_dir_all(&harness.root).ok();
}

fn evidence_ready(history: &DecisionHistorySnapshot) -> bool {
    history
        .records
        .iter()
        .find(|record| transform(record))
        .is_some_and(|terminal| {
            history
                .records
                .iter()
                .any(|record| later_cpu_price(record, terminal.sequence))
        })
}

fn transform(record: &DecisionRecord) -> bool {
    matches!(record.eventual_outcome, DecisionOutcome::Succeeded { bytes, .. } if bytes > 0)
        && matches!(
            record
                .warp_decision
                .as_ref()
                .and_then(|warp| warp.selected.as_ref())
                .map(|action| &action.command),
            Some(RecordedWarpCommand::Transform { .. })
        )
}

fn later_cpu_price(record: &DecisionRecord, sequence: u64) -> bool {
    record.sequence > sequence
        && record
            .warp_decision
            .as_ref()
            .is_some_and(|warp| warp.prices.cpu_micros > 0)
}
