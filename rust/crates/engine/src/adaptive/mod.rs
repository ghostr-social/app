//! Pure adaptive allocation policy for progressive media delivery.

mod admission;
mod allocation;
mod allocation_evidence;
mod allocation_geometry;
mod catalog_snapshot;
mod commitments;
mod current_lane;
mod eviction;
mod frontier;
mod navigation;
mod plan;
mod policy;
mod ranges;
mod reserve_evidence;
mod reserve_model;
mod reserve_origins;
mod reserve_schedule;
mod reserve_window;
mod reserves;
mod resources;
mod snapshot;
mod sources;

pub use allocation::REQUEST_SLICE_BYTES;
pub use catalog_snapshot::{candidate_snapshot, CandidateEvidence};
pub use navigation::{FeedOffset, NavigationDirection, NavigationHistory};
pub use plan::{
    Allocation, AllocationPlan, AllocationReason, CandidateUtility, ControlMode, DiscoveryDemand,
    Eviction, EvictionReason, NextReserveEvidence, NextReserveInfeasibility, PreemptionAuthority,
    ReadyReserveEvidence, ReserveCandidateEvidence, ReserveCandidateState, RetainedAllocation,
};
pub use policy::AdaptivePlayabilityPolicy;
pub use snapshot::{
    CandidateSnapshot, CurrentAuthority, InFlightRange, MediaLayout, NavigationSnapshot,
    NetworkSnapshot, OriginHealth, PlayabilitySnapshot, PlayableRange, PlaybackSnapshot,
    SnapshotError, StorageSnapshot, ViewProbability,
};
