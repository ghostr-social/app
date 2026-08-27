use super::readiness_support::readiness;
use crate::evaluation::EvaluationTracker;

#[test]
fn absent_outcomes_do_not_report_perfect_calibration() {
    let metrics = readiness(&EvaluationTracker::default());

    assert_eq!(metrics["on_time_readiness_samples"], 0);
    assert_eq!(metrics["on_time_readiness_expected_bps"], 0);
    assert_eq!(metrics["on_time_readiness_observed_bps"], 0);
    assert_eq!(metrics["on_time_readiness_calibration_error_bps"], 0);
    assert_eq!(metrics["on_time_readiness_calibration_bps"], 0);
}
