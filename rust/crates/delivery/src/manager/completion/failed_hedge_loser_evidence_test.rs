use super::{completion_use, CompletionUse};
use crate::manager::completion_observability::axiom_test_support::hedge_metric_fixture;
use crate::manager::inflight::CompletionStatus;

#[test]
fn failed_alternate_loser_is_retained_as_physical_origin_evidence() {
    let done = hedge_metric_fixture::failed();

    assert_eq!(
        completion_use(CompletionStatus::HedgeLoser, &done),
        CompletionUse::OriginEvidence
    );
}
