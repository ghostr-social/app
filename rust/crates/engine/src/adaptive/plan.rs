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
    MediaLayoutDiscovery,
    UsefulCommitment,
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
}
