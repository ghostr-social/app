//! One planning pass: on-disk byte ranges plus engine state in, the
//! ordered chunk-request list and per-post source URLs out.

use crate::manager::inflight::ActiveRange;
use crate::manager::retry::RetryBook;
use crate::manager::state::DeliveryState;
use crate::playback_demand::DemandSignal;
use ghostr_engine::adaptive::{DiscoveryDemand, Eviction, StorageSnapshot};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::scheduling::RangeRequest;
use ghostr_engine::{ByteRange, PostId};
use std::collections::{HashMap, HashSet};

mod adaptive;

/// Everything a planning pass reads besides the engine state.
pub(crate) struct PlanInputs<'a> {
    pub stats: &'a HostStats,
    pub retry: &'a RetryBook,
    pub present: &'a HashMap<PostId, Vec<ByteRange>>,
    pub in_flight: &'a [ActiveRange],
    pub storage: StorageSnapshot,
    pub connection_capacity: usize,
    pub connection_ceiling: usize,
    pub packet_loss_bps: u16,
    pub observed_at_ms: u64,
    pub demanded: Option<DemandSignal>,
}

pub(crate) struct PlannedWork {
    pub plan: ghostr_engine::adaptive::AllocationPlan,
    pub transfers: Vec<PlannedTransfer>,
    pub retained: HashSet<PlannedTransferId>,
    pub evictions: Vec<Eviction>,
    pub emergency: bool,
    pub discovery_demand: DiscoveryDemand,
}

#[derive(Clone)]
pub(crate) struct PlannedTransfer {
    pub request: RangeRequest,
    pub url: String,
    pub identity: TransferIdentity,
    pub commitment_until_ms: u64,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct PlannedTransferId {
    pub(crate) chunk: ghostr_engine::ChunkId,
    pub(crate) identity: TransferIdentity,
}

impl PlannedTransfer {
    pub(crate) fn id(&self) -> PlannedTransferId {
        PlannedTransferId {
            chunk: self.request.chunk.clone(),
            identity: self.identity.clone(),
        }
    }
}

/// Runs the pure engine planner over the manager's current picture.
/// Posts with no source left to try drop out entirely: they are
/// terminal, not work to reschedule on the next pass.
pub(crate) fn planned_work(state: &mut DeliveryState, inputs: PlanInputs<'_>) -> PlannedWork {
    adaptive::planned_work(state, inputs)
}
