use super::{hedge_metric_fixture, transfer_event};
use crate::manager::inflight::CompletionStatus;

#[test]
fn a_rejected_promotable_response_does_not_claim_an_avoided_restart() {
    let mut done = hedge_metric_fixture::done(0, true);
    let result = done.outcome.as_mut().expect("classified response");
    result.promoted = false;
    let event = transfer_event(&done, CompletionStatus::Current, None);

    assert!(!event.promotion_avoided_restart);
}
