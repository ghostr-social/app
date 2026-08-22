use super::{decision_outcome, hedge_metric_fixture};
use crate::manager::inflight::CompletionStatus;
use ghostr_engine::adaptive::DecisionOutcome;

#[test]
fn failed_hedge_loser_preserves_its_failure_outcome() {
    let outcome = decision_outcome(
        CompletionStatus::HedgeLoser,
        &hedge_metric_fixture::failed(),
    );

    assert!(matches!(outcome, DecisionOutcome::Failed { .. }));
}
