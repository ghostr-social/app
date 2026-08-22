use super::{completion_use, CompletionUse};
use crate::manager::completion_observability::tests::hedge_metric_fixture;
use crate::manager::inflight::CompletionStatus;

#[test]
fn manager_cancelled_loser_does_not_train_origin_success() {
    let done = hedge_metric_fixture::done(7, true);

    assert_eq!(
        completion_use(CompletionStatus::HedgeLoser, &done),
        CompletionUse::Discarded
    );
}
