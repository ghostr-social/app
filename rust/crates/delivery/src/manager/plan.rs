//! One planning pass: on-disk byte ranges plus engine state in, the
//! ordered chunk-request list and per-post source URLs out.

use crate::manager::inflight::ActiveAction;
use crate::manager::retry::RetryBook;
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{DiscoveryDemand, Eviction, RetrievalRequest, StorageSnapshot};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::scheduling::RangeRequest;
use ghostr_engine::{ActionId, ByteRange, PostId};
use ghostr_partial_store::partial_range_store::ContentRevision;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

mod adaptive;

/// Everything a planning pass reads besides the engine state.
pub(crate) struct PlanInputs<'a> {
    pub stats: &'a HostStats,
    pub retry: &'a RetryBook,
    pub present: &'a HashMap<PostId, Vec<ByteRange>>,
    pub finalized: &'a HashSet<PostId>,
    pub stored_totals: &'a HashMap<PostId, u64>,
    pub continuation_sources: &'a HashMap<PostId, String>,
    pub revisions: &'a HashMap<PostId, ContentRevision>,
    pub independent_sources: &'a HashMap<PostId, HashSet<String>>,
    pub completed_head_probes: &'a HashSet<PostId>,
    pub in_flight: &'a [ActiveAction],
    pub active_head_probes: &'a [TransferIdentity],
    pub storage: StorageSnapshot,
    pub connection_capacity: usize,
    pub connection_ceiling: usize,
    pub per_authority_request_limit: usize,
    pub packet_loss_bps: u16,
    pub observed_at_ms: u64,
    pub demanded: &'a HashMap<PostId, ByteRange>,
}

pub(crate) struct PlannedWork {
    pub plan: ghostr_engine::adaptive::AllocationPlan,
    pub transfers: Vec<PlannedTransfer>,
    pub selected_transfers: Vec<PlannedTransfer>,
    pub retained: HashSet<ActionId>,
    pub evictions: Vec<Eviction>,
    pub emergency: bool,
    pub discovery_demand: DiscoveryDemand,
    pub snapshot: Option<ghostr_engine::adaptive::PlayabilitySnapshot>,
    pub decision_models: Vec<ghostr_engine::adaptive::DecisionModelInput>,
    pub shadow_prices: ghostr_engine::adaptive::ShadowPrices,
    pub active_requests: u64,
    pub hedge_tails: Vec<crate::manager::hedge_tail::HedgeTailWake>,
    pub planner_cpu_micros: u64,
    pub warp: Option<ghostr_engine::adaptive::WarpPlanningDecision>,
}

#[derive(Clone)]
pub(crate) struct PlannedTransfer {
    pub request: RangeRequest,
    pub retrieval: RetrievalRequest,
    pub url: String,
    pub identity: TransferIdentity,
    pub commitment_until_ms: u64,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct PlannedTransferId {
    pub(crate) chunk: ghostr_engine::ChunkId,
    pub(crate) identity: TransferIdentity,
    pub(crate) retrieval: RetrievalRequest,
}

impl PlannedTransfer {
    pub(crate) fn id(&self) -> PlannedTransferId {
        PlannedTransferId {
            chunk: self.request.chunk.clone(),
            identity: self.identity.clone(),
            retrieval: self.retrieval,
        }
    }
}

/// Runs the pure engine planner over the manager's current picture.
/// Posts with no source left to try drop out entirely: they are
/// terminal, not work to reschedule on the next pass.
#[cfg(test)]
pub(crate) fn planned_work(state: &mut DeliveryState, inputs: PlanInputs<'_>) -> PlannedWork {
    let mut planner = ghostr_engine::adaptive::WarpPlanner::default();
    planned_work_with_planner(state, inputs, &mut planner)
}

pub(crate) fn planned_work_with_planner(
    state: &mut DeliveryState,
    inputs: PlanInputs<'_>,
    planner: &mut ghostr_engine::adaptive::WarpPlanner,
) -> PlannedWork {
    let started = Instant::now();
    let mut planned = adaptive::planned_work(state, inputs, planner);
    planned.planner_cpu_micros =
        started.elapsed().as_micros().clamp(1, u128::from(u64::MAX)) as u64;
    planned
}
