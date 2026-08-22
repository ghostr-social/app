use super::{decision_outcome, hedge_metric_fixture};
use crate::manager::inflight::CompletionStatus;
use ghostr_engine::adaptive::DecisionOutcome;

#[test]
fn ignored_range_is_a_typed_warp_failure() {
    let mut done = hedge_metric_fixture::done(0, false);
    let result = done.outcome.as_mut().expect("range response");
    result.range_ignored = true;
    result.range_support = Some(false);

    let outcome = decision_outcome(CompletionStatus::Current, &done);

    assert_eq!(
        outcome,
        DecisionOutcome::Failed {
            class: "warp_range_noncompliant".into(),
            elapsed_ms: 0,
        }
    );
}
