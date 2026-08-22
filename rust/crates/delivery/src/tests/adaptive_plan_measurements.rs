#[derive(Clone, Copy, Default)]
pub(super) struct PlanMeasurements {
    pub(super) network_bytes_per_second: u64,
    pub(super) capacity_revision: u64,
}

impl PlanMeasurements {
    pub(super) const fn new(network_bytes_per_second: u64, capacity_revision: u64) -> Self {
        Self {
            network_bytes_per_second,
            capacity_revision,
        }
    }
}
