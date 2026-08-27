use super::readiness_support::readiness;
use crate::evaluation::{EvaluationTracker, ReadinessMetricEvent};

#[test]
fn explicit_underflow_duration_does_not_relabel_the_whole_observed_interval() {
    let mut tracker = EvaluationTracker::default();
    tracker.readiness(ReadinessMetricEvent {
        observed_at_ms: 1_000,
        observed_ms: 1_000,
        underflow_ms: 250,
        ..ReadinessMetricEvent::default()
    });

    let metrics = readiness(&tracker);
    assert_eq!(metrics["observed_ms"], 1_000);
    assert_eq!(metrics["reserve_underflow_ms"], 250);
    assert_eq!(metrics["reserve_underflow_frequency_bps"], 2_500);
}
