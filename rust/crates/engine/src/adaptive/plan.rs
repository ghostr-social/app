use super::RetrievalRequest;
use crate::{ActionId, ByteRange, PostId};
use serde::{Deserialize, Serialize};

mod replay;
mod reserve;
pub use reserve::{
    ControlMode, NextReserveEvidence, NextReserveInfeasibility, ReadyReserveEvidence,
    ReserveCandidateEvidence, ReserveCandidateKind, ReserveCandidateState,
};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub struct CandidateUtility {
    pub view_probability: f64,
    pub additional_playable_ms: u64,
    pub expected_delivery_ms: u64,
    pub score: f64,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum PreemptionAuthority {
    PlaybackCritical,
    Transition,
    Speculative,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum AllocationReason {
    CurrentStallPrevention,
    CurrentBufferReserve,
    LikelyNextTransition,
    RapidNavigationCoverage,
    MediaBootstrap,
    MediaLayoutDiscovery,
    NextStartability,
    UsefulCommitment,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum DiscoveryDemand {
    Expand,
    #[default]
    Hold,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Allocation {
    pub post: PostId,
    pub request: RetrievalRequest,
    pub source: String,
    pub expected_playable_gain_ms: u64,
    pub utility: CandidateUtility,
    pub authority: PreemptionAuthority,
    pub commitment_until_ms: u64,
    pub reason: AllocationReason,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct RetainedAllocation {
    pub action_id: ActionId,
    pub post: PostId,
    pub request: RetrievalRequest,
    pub source: String,
    pub utility: CandidateUtility,
    pub committed_until_ms: u64,
    pub reason: AllocationReason,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum EvictionReason {
    StoragePressure,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Eviction {
    pub post: PostId,
    pub range: ByteRange,
    pub expected_playable_loss_ms: f64,
    pub reason: EvictionReason,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct AllocationPlan {
    pub allocations: Vec<Allocation>,
    pub retained: Vec<RetainedAllocation>,
    pub evictions: Vec<Eviction>,
    pub discovery_demand: DiscoveryDemand,
    pub mode: ControlMode,
    pub ready_reserve: ReadyReserveEvidence,
    pub next_reserve: NextReserveEvidence,
}
