use core::num::{NonZeroU128, NonZeroU16};
use core::time::Duration;

const NORMAL_RATE: NonZeroU16 = NonZeroU16::new(1_000).expect("positive normal playback rate");

/// Elapsed media time and the current presentation's selected playback rate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WatchProgress {
    elapsed: Duration,
    rate_milli: NonZeroU16,
}

impl WatchProgress {
    pub const fn normal(elapsed: Duration) -> Self {
        Self::at_rate(elapsed, NORMAL_RATE)
    }

    pub const fn at_rate(elapsed: Duration, rate_milli: NonZeroU16) -> Self {
        Self {
            elapsed,
            rate_milli,
        }
    }

    pub(super) fn elapsed_ms(self) -> u64 {
        self.elapsed.as_millis().min(u128::from(u64::MAX)) as u64
    }

    pub(super) fn is_normal_start(self) -> bool {
        self.elapsed_ms() == 0 && self.rate_milli == NORMAL_RATE
    }

    pub(super) fn wall_time_ms(self, media_ms: u64) -> u64 {
        let millis = u128::from(media_ms) * 1_000 / NonZeroU128::from(self.rate_milli);
        millis.min(u128::from(u64::MAX)) as u64
    }
}

impl Default for WatchProgress {
    fn default() -> Self {
        Self::normal(Duration::ZERO)
    }
}
