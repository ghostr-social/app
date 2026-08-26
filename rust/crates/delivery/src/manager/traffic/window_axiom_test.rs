use super::*;

impl OverallTrafficWindow {
    pub(crate) fn bytes(self) -> u64 {
        self.bytes
    }
    pub(crate) fn elapsed(self) -> Duration {
        self.elapsed
    }
    pub(crate) fn observed_at_ms(self) -> u64 {
        self.observed_at_ms
    }
}
