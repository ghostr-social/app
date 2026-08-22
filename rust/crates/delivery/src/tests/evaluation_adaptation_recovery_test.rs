use crate::evaluation::{AdaptationMetricEvent, EvaluationTracker};

#[test]
fn adaptation_records_one_change_recovery_and_explicit_calibration() {
    let mut tracker = EvaluationTracker::default();
    tracker.adaptation(event(1_000, true, true, [false, true, true]));
    tracker.adaptation(event(1_100, true, false, [true, true, true]));
    tracker.adaptation(event(1_400, false, true, [true, true, true]));

    let metrics = tracker.snapshot().adaptation;
    assert_eq!(metrics.origin_change_points, 1);
    assert_eq!(metrics.recovery_after_change.samples, 1);
    assert_eq!(metrics.recovery_after_change.p50_ms, 400);
    assert_eq!(metrics.success_predictions, 3);
    assert_eq!(metrics.success_observed_bps, 6_667);
    assert_eq!(metrics.success_calibration_error_bps, 1_333);
    assert_eq!(metrics.latency_p50_coverage_bps, 6_667);
    assert_eq!(metrics.latency_p95_coverage_bps, 10_000);
    assert_eq!(metrics.latency_p99_coverage_bps, 10_000);
}

fn event(
    observed_at_ms: u64,
    adapting: bool,
    succeeded: bool,
    latency: [bool; 3],
) -> AdaptationMetricEvent {
    AdaptationMetricEvent {
        origin: "media.example".into(),
        observed_at_ms,
        adapting,
        predicted_success_bps: 8_000,
        succeeded: Some(succeeded),
        latency_quantiles_on_time: Some(latency),
        ..AdaptationMetricEvent::default()
    }
}
