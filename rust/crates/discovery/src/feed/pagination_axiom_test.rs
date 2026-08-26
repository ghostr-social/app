use super::*;

/// Inclusive `until` cutoff from a UTC unix-millisecond clock value.
pub(crate) fn older_than_from_unix_millis(millis: u64) -> Timestamp {
    Timestamp::from(millis / 1000)
}
