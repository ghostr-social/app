use super::{decision_outcome, hedge_metric_fixture, transfer_event};
use crate::manager::inflight::CompletionStatus;
use ghostr_engine::adaptive::DecisionOutcome;

#[test]
fn discarded_policy_bytes_are_network_waste_not_storage_occupancy() {
    let done = hedge_metric_fixture::policy_limited();
    let resolution = hedge_metric_fixture::resolution("whole");

    let event = transfer_event(&done, CompletionStatus::Current, Some(&resolution));

    assert_eq!(event.total_bytes, 9);
    assert_eq!(event.aborted_bytes, 9);
    assert_eq!(event.storage_byte_ms, 0);
}

#[test]
fn focus_cancellation_does_not_mask_a_typed_policy_stop() {
    let outcome = decision_outcome(
        CompletionStatus::Cancelled,
        &hedge_metric_fixture::policy_limited(),
    );

    assert!(matches!(outcome, DecisionOutcome::Failed { class, .. }
        if class == "warp_whole_body_limit"));
}
