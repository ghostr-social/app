use crate::evaluation::{BudgetMetricEvent, EvaluationTracker};

#[test]
fn storage_time_integrates_observed_resident_bytes() {
    let mut tracker = EvaluationTracker::default();
    tracker.budget(sample(1_000, 10));
    tracker.budget(sample(1_500, 20));
    tracker.budget(sample(1_600, 30));

    assert_eq!(tracker.snapshot().efficiency.storage_byte_ms, 7_000);
}

fn sample(observed_at_ms: u64, stored_bytes: u64) -> BudgetMetricEvent {
    BudgetMetricEvent {
        observed_at_ms,
        stored_bytes,
        ..BudgetMetricEvent::default()
    }
}
