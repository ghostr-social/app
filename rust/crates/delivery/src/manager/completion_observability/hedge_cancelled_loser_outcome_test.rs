use super::{decision_outcome, hedge_metric_fixture};
use crate::manager::inflight::CompletionStatus;
use ghostr_engine::adaptive::DecisionOutcome;

#[test]
fn cancelled_hedge_loser_resolves_as_cancelled() {
    let done = hedge_metric_fixture::done(7, true);

    assert_eq!(
        decision_outcome(CompletionStatus::HedgeLoser, &done),
        DecisionOutcome::Cancelled {
            bytes: 7,
            elapsed_ms: 0,
        }
    );
}
