use super::*;

impl OriginObservation {
    pub fn with_range_compliance(mut self, value: bool) -> Self {
        self.range_compliant = Some(value);
        self
    }

    pub fn with_throughput_bps(mut self, value: u64) -> Self {
        self.throughput_bps = Some(value.max(1));
        self
    }
}
