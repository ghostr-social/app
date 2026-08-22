use crate::manager::traffic::SAMPLE_INTERVAL;

pub(crate) fn new_at(origin: tokio::time::Instant) -> tokio::time::Interval {
    let start = origin + SAMPLE_INTERVAL;
    let mut interval = tokio::time::interval_at(start, SAMPLE_INTERVAL);
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    interval
}

#[cfg(test)]
pub(crate) fn new() -> tokio::time::Interval {
    new_at(tokio::time::Instant::now())
}
