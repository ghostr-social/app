use super::readiness_support::{observe, readiness};
use crate::evaluation::EvaluationTracker;

#[test]
fn inferred_duration_belongs_to_the_state_that_covered_the_interval() {
    let mut tracker = EvaluationTracker::default();
    observe(&mut tracker, 0, false);
    observe(&mut tracker, 100, true);
    observe(&mut tracker, 1_100, false);

    let metrics = readiness(&tracker);
    assert_eq!(metrics["reserve_underflows"], 1);
    assert_eq!(metrics["observed_ms"], 1_100);
    assert_eq!(metrics["reserve_underflow_ms"], 1_000);
    assert_eq!(metrics["reserve_underflow_frequency_bps"], 9_090);
    assert_eq!(metrics["replenish_after_burst"]["samples"], 1);
    assert_eq!(metrics["replenish_after_burst"]["p50_ms"], 1_000);
}
