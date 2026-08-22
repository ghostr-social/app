use crate::adaptive::{CompletionTimes, DeadlineReadiness, PlannerWatchEvidence};

pub(super) fn readiness(
    fallback: u64,
    timing: CompletionTimes,
    success: u16,
    watch: PlannerWatchEvidence,
) -> Vec<DeadlineReadiness> {
    deadline_values(fallback, watch)
        .into_iter()
        .map(|deadline| DeadlineReadiness::new(deadline, probability(deadline, timing, success)))
        .collect()
}

fn deadline_values(fallback: u64, watch: PlannerWatchEvidence) -> Vec<u64> {
    let Some((quantiles, safety)) = watch.deadlines() else {
        return vec![fallback];
    };
    let mut values = quantiles
        .into_iter()
        .filter(|deadline| *deadline > 0)
        .collect::<Vec<_>>();
    values.extend(safety);
    if values.is_empty() {
        values.push(fallback);
    }
    values.sort_unstable();
    values.dedup();
    values
}

fn probability(deadline: u64, timing: CompletionTimes, success: u16) -> u16 {
    if timing.p95_ms <= deadline {
        return success.saturating_mul(95) / 100;
    }
    if timing.expected_ms <= deadline {
        return success / 2;
    }
    0
}
