#![cfg(unix)]

mod delivery_fixture;
mod focus_wait_fixture;
mod priced_transform_delivery_fixture;
mod priced_transform_fixture;
mod transform_delivery_fixture;

use delivery_fixture::decision::wait_for_history;
use delivery_fixture::evidence::DeliveryEvidence as _;
use ghostr_delivery::delivery_events::DecisionHistorySnapshot;
use ghostr_engine::adaptive::{DecisionOutcome, DecisionRecord, RecordedWarpCommand};
use priced_transform_fixture::PricedRemux;
use std::sync::Arc;

#[tokio::test]
async fn measured_transform_records_resources_without_enabling_core_cpu_prices() {
    let harness =
        priced_transform_delivery_fixture::start("warp-transform-cpu-price", Arc::new(PricedRemux))
            .await;
    wait_for_history(&harness.handle, evidence_ready).await;

    let history = harness.handle.decision_history();
    let terminal = history
        .records
        .iter()
        .find(|record| transform(record))
        .expect("valid test fixture");
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
        .find(|record| later_core_decision(record, terminal.sequence))
        .expect("valid test fixture");
    assert_eq!(
        later
            .warp_decision
            .as_ref()
            .expect("valid test fixture")
            .prices
            .cpu_micros,
        0
    );
    harness.handle.clear().await.expect("valid test fixture");
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
                .any(|record| later_core_decision(record, terminal.sequence))
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

fn later_core_decision(record: &DecisionRecord, sequence: u64) -> bool {
    record.sequence > sequence
        && record
            .warp_decision
            .as_ref()
            .is_some_and(|warp| warp.prices.cpu_micros == 0)
}
