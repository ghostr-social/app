use super::{hedge_metric_fixture, transfer_event};
use crate::manager::inflight::CompletionStatus;

#[test]
fn successful_alternate_winner_bytes_are_useful() {
    let done = hedge_metric_fixture::done(16, false);
    let resolution = hedge_metric_fixture::resolution("hedge");

    let event = transfer_event(&done, CompletionStatus::HedgeWinner, Some(&resolution));

    assert_eq!(event.total_bytes, 16);
    assert_eq!(event.duplicate_hedge_bytes, 0);
    assert_eq!(event.aborted_bytes, 0);
}
