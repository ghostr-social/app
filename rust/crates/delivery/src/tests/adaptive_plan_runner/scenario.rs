use crate::manager::inflight::ActiveAction;
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::StorageSnapshot;
use ghostr_engine::host_stats::ThroughputSample;
use std::collections::HashMap;
use std::time::Duration;

pub(in crate::tests) struct PlanScenario<'a> {
    pub(in crate::tests) state: DeliveryState,
    pub(in crate::tests) buffer_ms: u64,
    pub(in crate::tests) bytes_per_second: u64,
    pub(in crate::tests) storage: StorageSnapshot,
    pub(in crate::tests) present: HashMap<ghostr_engine::PostId, Vec<ghostr_engine::ByteRange>>,
    pub(in crate::tests) packet_loss_bps: u16,
    pub(in crate::tests) in_flight: &'a [ActiveAction],
    pub(in crate::tests) connection_capacity: usize,
}

impl PlanScenario<'_> {
    pub(super) fn throughput_sample(&self) -> ThroughputSample {
        ThroughputSample::new(
            self.bytes_per_second,
            Duration::from_secs(1),
            1_000,
            self.connection_capacity,
        )
        .expect("valid throughput sample")
    }
}
