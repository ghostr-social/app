use super::{hedge_metric_fixture, transfer_event};
use crate::manager::inflight::CompletionStatus;

#[test]
fn admitted_promotion_counts_the_restart_it_avoided_before_later_cancellation() {
    let mut done = hedge_metric_fixture::done(7, true);
    done.outcome.as_mut().expect("admitted response").promoted = true;
    let event = transfer_event(&done, CompletionStatus::Cancelled, None);

    assert!(event.promotion_avoided_restart);
}
