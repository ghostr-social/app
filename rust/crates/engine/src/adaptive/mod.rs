//! Pure adaptive allocation policy for progressive media delivery.

mod admission;
mod allocation;
mod allocation_evidence;
mod catalog_snapshot;
mod commitments;
mod eviction;
mod navigation;
mod plan;
mod policy;
mod ranges;
mod resources;
mod snapshot;
mod sources;

pub use allocation::REQUEST_SLICE_BYTES;
pub use catalog_snapshot::{candidate_snapshot, CandidateEvidence};
pub use navigation::{FeedOffset, NavigationDirection, NavigationHistory};
pub use plan::{
    Allocation, AllocationPlan, AllocationReason, CandidateUtility, DiscoveryDemand, Eviction,
    EvictionReason, PreemptionAuthority, RetainedAllocation,
};
pub use policy::AdaptivePlayabilityPolicy;
pub use snapshot::{
    CandidateSnapshot, InFlightRange, MediaLayout, NavigationSnapshot, NetworkSnapshot,
    OriginHealth, PlayabilitySnapshot, PlayableRange, PlaybackSnapshot, SnapshotError,
    StorageSnapshot, ViewProbability,
};
