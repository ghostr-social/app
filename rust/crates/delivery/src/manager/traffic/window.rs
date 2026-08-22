use std::time::Duration;
use tokio::time::Instant;

#[derive(Clone, Copy)]
pub(crate) struct TrafficWindow {
    ended: Instant,
}

impl TrafficWindow {
    pub(crate) fn new(_started: Instant, ended: Instant) -> Self {
        Self { ended }
    }

    pub(super) fn ended(self) -> Instant {
        self.ended
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct OverallTrafficWindow {
    bytes: u64,
    elapsed: Duration,
    peak_active_transfers: usize,
    observed_at_ms: u64,
    latest_ttfb: Option<Duration>,
}

impl OverallTrafficWindow {
    pub(crate) fn new(
        bytes: u64,
        elapsed: Duration,
        peak_active_transfers: usize,
        observed_at_ms: u64,
        latest_ttfb: Option<Duration>,
    ) -> Self {
        Self {
            bytes,
            elapsed,
            peak_active_transfers,
            observed_at_ms,
            latest_ttfb,
        }
    }

    #[cfg(test)]
    pub(crate) fn bytes(self) -> u64 {
        self.bytes
    }

    #[cfg(test)]
    pub(crate) fn elapsed(self) -> Duration {
        self.elapsed
    }

    pub(crate) fn bytes_per_second(self) -> f64 {
        self.bytes as f64 / self.elapsed.as_secs_f64()
    }

    pub(crate) fn peak_active_transfers(self) -> usize {
        self.peak_active_transfers
    }

    #[cfg(test)]
    pub(crate) fn observed_at_ms(self) -> u64 {
        self.observed_at_ms
    }

    pub(crate) fn latest_ttfb(self) -> Option<Duration> {
        self.latest_ttfb
    }
}
