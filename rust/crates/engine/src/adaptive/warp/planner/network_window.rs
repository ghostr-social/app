use super::WarpPlanner;

impl WarpPlanner {
    /// Consumes the same rate allowance used by new actions before a body resumes.
    pub fn reserve_network_window(&mut self, bytes: u64, now_ms: u64) -> bool {
        bytes <= crate::adaptive::REQUEST_SLICE_BYTES
            && self.network.as_mut().is_some_and(|bucket| bucket.consume(bytes, now_ms))
    }

    pub fn network_window_deadline_ms(&mut self, bytes: u64, now_ms: u64) -> Option<u64> {
        self.network.as_mut()?.refill_deadline_ms(bytes, now_ms)
    }
}
