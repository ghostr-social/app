use crate::manager::traffic::SAMPLE_INTERVAL;

pub(super) fn new_at(origin: tokio::time::Instant) -> tokio::time::Interval {
    let start = origin + SAMPLE_INTERVAL;
    let mut interval = tokio::time::interval_at(start, SAMPLE_INTERVAL);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    interval
}

#[cfg(test)]
#[path = "control_interval_axiom_test.rs"]
pub(crate) mod axiom_test_support;
