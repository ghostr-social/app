use super::readiness_support::{observe, readiness};
use crate::evaluation::{EvaluationTracker, ReadinessMetricEvent};

#[test]
fn explicit_replenishment_latency_replaces_the_inferred_transition_sample() {
    let mut tracker = EvaluationTracker::default();
    observe(&mut tracker, 0, true);
    tracker.readiness(ReadinessMetricEvent {
        observed_at_ms: 1_000,
        underflow: false,
        replenished_after_ms: Some(900),
        ..ReadinessMetricEvent::default()
    });

    let metrics = readiness(&tracker);
    assert_eq!(metrics["reserve_underflow_ms"], 1_000);
    assert_eq!(metrics["replenish_after_burst"]["samples"], 1);
    assert_eq!(metrics["replenish_after_burst"]["p50_ms"], 900);
}
