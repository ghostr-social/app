use crate::manager::inflight::ActiveAction;
use crate::manager::state::DeliveryState;
use core::time::Duration;
use ghostr_engine::adaptive::StorageSnapshot;
use ghostr_engine::host_stats::ThroughputSample;
use std::collections::HashMap;

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

#[derive(Clone, Copy, Default)]
pub(super) struct PlanRunOptions<'a> {
    pub(super) watch: Option<&'a ghostr_engine::watch_model::WatchModel>,
    pub(super) per_authority_limit: Option<usize>,
    pub(super) hls: Option<&'a [ghostr_engine::adaptive::HlsCandidateSnapshot]>,
}

impl<'a> PlanRunOptions<'a> {
    pub(super) fn with_watch(mut self, model: &'a ghostr_engine::watch_model::WatchModel) -> Self {
        self.watch = Some(model);
        self
    }

    pub(super) const fn with_per_authority_limit(mut self, limit: usize) -> Self {
        self.per_authority_limit = Some(limit);
        self
    }

    pub(super) const fn with_hls(
        mut self,
        candidates: &'a [ghostr_engine::adaptive::HlsCandidateSnapshot],
    ) -> Self {
        self.hls = Some(candidates);
        self
    }
}
