use super::readiness_support::{observe, readiness};
use crate::evaluation::EvaluationTracker;

#[test]
fn a_regressed_timestamp_cannot_inflate_the_next_interval() {
    let mut tracker = EvaluationTracker::default();
    observe(&mut tracker, 1_000, false);
    observe(&mut tracker, 900, true);
    observe(&mut tracker, 1_100, false);

    let metrics = readiness(&tracker);
    assert_eq!(metrics["observed_ms"], 100);
    assert_eq!(metrics["reserve_underflow_ms"], 100);
    assert_eq!(metrics["replenish_after_burst"]["samples"], 1);
    assert_eq!(metrics["replenish_after_burst"]["p50_ms"], 100);
}
