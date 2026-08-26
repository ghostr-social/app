#![cfg(unix)]

mod delivery_fixture;
mod focus_wait_fixture;
mod priced_transform_delivery_fixture;
mod rejected_transform_fixture;
mod transform_delivery_fixture;

use delivery_fixture::decision::wait_for_history;
use delivery_fixture::evidence::DeliveryEvidence as _;
use ghostr_engine::adaptive::{DecisionOutcome, DecisionRecord, RecordedWarpCommand};
use rejected_transform_fixture::RejectedRemux;
use std::sync::Arc;

#[tokio::test]
async fn measured_backend_failure_stays_failed_with_cpu_and_no_storage() {
    let harness = priced_transform_delivery_fixture::start(
        "warp-transform-failed-resources",
        Arc::new(RejectedRemux),
    )
    .await;
    wait_for_history(&harness.handle, |history| {
        history.records.iter().any(failed_transform)
    })
    .await;

    let history = harness.handle.decision_history();
    let record = history
        .records
        .iter()
        .find(|record| failed_transform(record))
        .expect("valid test fixture");
    let actual = record.actual_resources.expect("measured failure resources");
    assert!(actual.cpu_ms > 0, "failed transform CPU must be measured");
    assert_eq!(
        actual.storage_bytes, 0,
        "a rejected transform must publish no storage bytes"
    );
    assert!(
        matches!(&record.eventual_outcome, DecisionOutcome::Failed { class, .. }
        if class == "warp_transform_backend_rejected"),
        "the decision must retain the backend-rejection class"
    );
    harness.handle.clear().await.expect("valid test fixture");
    std::fs::remove_dir_all(&harness.root).ok();
}

fn failed_transform(record: &DecisionRecord) -> bool {
    matches!(record.eventual_outcome, DecisionOutcome::Failed { .. })
        && matches!(
            record
                .warp_decision
                .as_ref()
                .and_then(|warp| warp.selected.as_ref())
                .map(|action| &action.command),
            Some(RecordedWarpCommand::Transform { .. })
        )
}
