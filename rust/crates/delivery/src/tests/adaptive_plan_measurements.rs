#[derive(Clone, Copy, Default)]
pub(super) struct PlanMeasurements {
    network_bytes_per_second: u64,
    pub(super) capacity_revision: u64,
}

impl PlanMeasurements {
    pub(super) const fn new(network_bytes_per_second: u64, capacity_revision: u64) -> Self {
        Self {
            network_bytes_per_second,
            capacity_revision,
        }
    }

    pub(super) const fn feedback(
        self,
        storage: ghostr_engine::adaptive::StorageSnapshot,
        network_target: u64,
        request_target: u64,
    ) -> ghostr_engine::adaptive::ResourceFeedback {
        ghostr_engine::adaptive::ResourceFeedback {
            revision: 1,
            actual: ghostr_engine::adaptive::ResourceObservation::new(
                self.network_bytes_per_second,
                storage.used_bytes,
                0,
                0,
            ),
            target: ghostr_engine::adaptive::ResourceObservation::new(
                network_target,
                storage.budget_bytes.saturating_mul(9) / 10,
                0,
                request_target,
            ),
            price_snapshot: None,
        }
    }
}
