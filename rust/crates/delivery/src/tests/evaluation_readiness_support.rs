use crate::evaluation::{EvaluationTracker, ReadinessMetricEvent};

pub(super) fn observe(tracker: &mut EvaluationTracker, observed_at_ms: u64, underflow: bool) {
    tracker.readiness(ReadinessMetricEvent {
        observed_at_ms,
        underflow,
        ..ReadinessMetricEvent::default()
    });
}

pub(super) fn readiness(tracker: &EvaluationTracker) -> serde_json::Value {
    serde_json::to_value(tracker.snapshot()).expect("evaluation snapshot")["readiness"].clone()
}
