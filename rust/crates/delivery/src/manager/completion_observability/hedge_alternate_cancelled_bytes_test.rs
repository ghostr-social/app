use super::{hedge_metric_fixture, transfer_event};
use crate::manager::inflight::CompletionStatus;

#[test]
fn cancelled_alternate_loser_bytes_remain_duplicate() {
    let done = hedge_metric_fixture::done(5, true);
    let resolution = hedge_metric_fixture::resolution("hedge");

    let event = transfer_event(&done, CompletionStatus::HedgeLoser, Some(&resolution));

    assert_eq!(event.duplicate_hedge_bytes, 5);
}
