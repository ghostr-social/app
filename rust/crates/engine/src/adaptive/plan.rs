use crate::{ByteRange, PostId};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CandidateUtility {
    pub view_probability: f64,
    pub additional_playable_ms: u64,
    pub expected_delivery_ms: u64,
    pub score: f64,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PreemptionAuthority {
    PlaybackCritical,
    Transition,
    Speculative,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NextReserveInfeasibility {
    CurrentUnprotected,
    NoLiveOrigin,
    PolicyDenied,
    NoTransferBudget,
    NoStorageCapacity,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ControlMode {
    Emergency,
    Safety,
    #[default]
    Normal,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum ReserveCandidateState {
    #[default]
    Unprepared,
    Ready,
    InFlight,
    Probing,
    Preparing {
        ranges: Vec<ByteRange>,
    },
    Planned {
        ranges: Vec<ByteRange>,
    },
    Infeasible {
        reason: NextReserveInfeasibility,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReserveCandidateEvidence {
    pub post: PostId,
    pub state: ReserveCandidateState,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReadyReserveEvidence {
    pub target: usize,
    pub ready: usize,
    pub protected: usize,
    pub recovery_horizon_ms: u64,
    pub underflow_risk_bps: u16,
    pub ready_coverage_ms: u64,
    pub candidates: Vec<ReserveCandidateEvidence>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum NextReserveEvidence {
    #[default]
    NotApplicable,
    Ready {
        post: PostId,
    },
    InFlight {
        post: PostId,
    },
    Granted {
        post: PostId,
        range: ByteRange,
    },
    Infeasible {
        post: PostId,
        reason: NextReserveInfeasibility,
    },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DiscoveryDemand {
    Expand,
    #[default]
    Hold,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Allocation {
    pub post: PostId,
    pub range: ByteRange,
    pub source: String,
    pub expected_playable_gain_ms: u64,
    pub utility: CandidateUtility,
    pub authority: PreemptionAuthority,
    pub commitment_until_ms: u64,
    pub reason: AllocationReason,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RetainedAllocation {
    pub post: PostId,
    pub range: ByteRange,
    pub source: String,
    pub utility: CandidateUtility,
    pub committed_until_ms: u64,
    pub reason: AllocationReason,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvictionReason {
    StoragePressure,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Eviction {
    pub post: PostId,
    pub range: ByteRange,
    pub expected_playable_loss_ms: f64,
    pub reason: EvictionReason,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AllocationPlan {
    pub allocations: Vec<Allocation>,
    pub retained: Vec<RetainedAllocation>,
    pub evictions: Vec<Eviction>,
    pub discovery_demand: DiscoveryDemand,
    pub mode: ControlMode,
    pub ready_reserve: ReadyReserveEvidence,
    pub next_reserve: NextReserveEvidence,
}
