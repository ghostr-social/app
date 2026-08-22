use super::{hedge_metric_fixture, transfer_event};
use crate::manager::inflight::CompletionStatus;

#[test]
fn cancelled_primary_loser_bytes_are_duplicate() {
    let done = hedge_metric_fixture::done(7, true);
    let resolution = hedge_metric_fixture::resolution("transfer");

    let event = transfer_event(&done, CompletionStatus::HedgeLoser, Some(&resolution));

    assert_eq!(event.duplicate_hedge_bytes, 7);
    assert_eq!(event.aborted_bytes, 7);
}
