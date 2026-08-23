use super::{hedge_metric_fixture, transfer_event};
use crate::manager::inflight::CompletionStatus;

#[test]
fn network_metrics_include_received_bytes_rejected_by_the_store() {
    let mut done = hedge_metric_fixture::done(7, true);
    done.received_bytes = 9;
    let resolution = hedge_metric_fixture::resolution("transfer");

    let event = transfer_event(&done, CompletionStatus::HedgeLoser, Some(&resolution));

    assert_eq!(event.total_bytes, 9);
    assert_eq!(event.aborted_bytes, 9);
    assert_eq!(event.duplicate_hedge_bytes, 9);
    assert_eq!(event.storage_byte_ms, 7);
}
