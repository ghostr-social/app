use crate::evaluation::{EvaluationTracker, ReadinessMetricEvent};

#[test]
fn readiness_exports_predicted_observed_and_calibration_error() {
    let mut tracker = EvaluationTracker::default();
    tracker.readiness(event(1_000, 8_000, true));
    tracker.readiness(event(1_100, 6_000, false));

    let metrics = tracker.snapshot().readiness;
    assert_eq!(metrics.on_time_readiness_samples, 2);
    assert_eq!(metrics.on_time_readiness_expected_bps, 7_000);
    assert_eq!(metrics.on_time_readiness_observed_bps, 5_000);
    assert_eq!(metrics.on_time_readiness_calibration_error_bps, 2_000);
}

fn event(observed_at_ms: u64, predicted_bps: u16, observed: bool) -> ReadinessMetricEvent {
    ReadinessMetricEvent {
        observed_at_ms,
        on_time_prediction_bps: Some(predicted_bps),
        on_time_observed: Some(observed),
        ..ReadinessMetricEvent::default()
    }
}
