use super::{rounded_ms, BufferScenario};

impl BufferScenario {
    /// A distinct continuous-service model; never substitute it for batches
    /// whose bytes become playable only after a segment or block completes.
    pub(crate) fn continuous_required_ms(self, service_rate_milli: u64) -> u64 {
        let net_rate = u64::from(self.rate_milli).saturating_sub(service_rate_milli);
        rounded_ms(u128::from(self.horizon_ms) * u128::from(net_rate))
    }
}
