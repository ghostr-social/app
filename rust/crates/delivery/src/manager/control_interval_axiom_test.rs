use super::*;

pub(crate) fn new() -> tokio::time::Interval {
    new_at(tokio::time::Instant::now())
}
