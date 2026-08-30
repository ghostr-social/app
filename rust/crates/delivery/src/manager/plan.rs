//! One planning pass: on-disk byte ranges plus engine state in, the
//! ordered chunk-request list and per-post source URLs out.

use crate::manager::inflight::ActiveAction;
use crate::manager::retry::RetryBook;
use crate::manager::state::DeliveryState;
use ghostr_engine::adaptive::{
    ControlMode, DiscoveryDemand, Eviction, RetrievalRequest, StorageSnapshot, WholeBodyExhaustion,
};
use ghostr_engine::host_stats::HostStats;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::scheduling::RangeRequest;
use ghostr_engine::{ActionId, ByteRange, PostId};
use ghostr_partial_store::partial_range_store::ContentRevision;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

mod adaptive;

/// Everything a planning pass reads besides the engine state.
#[derive(Clone, Copy)]
pub(crate) struct PlanInputs<'a> {
    pub stats: &'a HostStats,
    pub retry: &'a RetryBook,
    pub present: &'a HashMap<PostId, Vec<ByteRange>>,
    pub finalized: &'a HashSet<PostId>,
    pub stored_totals: &'a HashMap<PostId, u64>,
    pub continuation_sources: &'a HashMap<PostId, String>,
    pub revisions: &'a HashMap<PostId, ContentRevision>,
    pub independent_sources: &'a HashMap<PostId, HashSet<String>>,
    pub whole_body_exhaustions: &'a HashMap<TransferIdentity, WholeBodyExhaustion>,
    pub completed_head_probes: &'a HashSet<TransferIdentity>,
    pub unavailable_head_probes: &'a HashSet<TransferIdentity>,
    pub in_flight: &'a [ActiveAction],
    pub active_head_probes: &'a [TransferIdentity],
    pub hls_candidates: &'a [ghostr_engine::adaptive::HlsCandidateSnapshot],
    pub active_hls_sources: &'a [String],
    pub segmented_storage_available_bytes: u64,
    pub storage: StorageSnapshot,
    pub connection_capacity: usize,
    pub hls_demand_expansion_allowed: bool,
    pub connection_ceiling: usize,
    pub per_authority_request_limit: usize,
    pub packet_loss_bps: u16,
    pub resource_feedback: Option<ghostr_engine::adaptive::ResourceFeedback>,
    pub capacity_revision: u64,
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
    pub network_refill_deadline_ms: Option<u64>,
    pub planner_cpu_micros: u64,
    pub warp: Option<ghostr_engine::adaptive::WarpPlanningDecision>,
    pub(crate) player_preparations: Vec<crate::delivery_events::PlayerPreparationClaim>,
}

impl PlannedWork {
    fn player_verified_posts(&self) -> Vec<PostId> {
        self.snapshot
            .as_ref()
            .into_iter()
            .flat_map(|snapshot| &snapshot.candidates)
            .filter(|candidate| {
                candidate.player_preparation
                    == ghostr_engine::adaptive::PlayerPreparation::FirstFrameRendered
            })
            .map(|candidate| candidate.post.clone())
            .collect()
    }
}

#[derive(Clone)]
pub(crate) struct PlannedTransfer {
    pub request: RangeRequest,
    pub retrieval: RetrievalRequest,
    pub control_mode: ControlMode,
    pub url: String,
    pub identity: TransferIdentity,
    pub profile: ghostr_engine::origin_model::OriginAttemptProfile,
    pub commitment_until_ms: u64,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct PlannedTransferId {
    chunk: ghostr_engine::ChunkId,
    identity: TransferIdentity,
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

pub(crate) fn planned_work_with_planner(
    state: &DeliveryState,
    inputs: &PlanInputs<'_>,
    planner: &mut ghostr_engine::adaptive::WarpPlanner,
    watch: &ghostr_engine::watch_model::WatchModel,
) -> PlannedWork {
    let started = Instant::now();
    let mut planned = adaptive::planned_work(state, inputs, planner, watch);
    let verified = planned.player_verified_posts();
    planned.player_preparations = state.player_preparation_claims(&verified);
    planned.planner_cpu_micros =
        started.elapsed().as_micros().clamp(1, u128::from(u64::MAX)) as u64;
    planned
}

#[cfg(test)]
#[path = "plan_axiom_test.rs"]
pub(crate) mod axiom_test_support;
