//! Pure adaptive allocation policy for progressive media delivery.

/// Initial cold-start whole-object discovery proposal (§5.5).
pub const BOOTSTRAP_DIRECT_FETCH_BYTES: u64 = 1024 * 1024;

/// Initial bounded prefix probe for unresolved progressive media (§5.5).
pub const MEDIA_BOOTSTRAP_PROBE_BYTES: u64 = 64 * 1024;

mod admission;
mod allocation;
mod allocation_evidence;
mod allocation_geometry;
mod catalog_snapshot;
mod commitments;
mod current_lane;
mod decision;
mod eviction;
mod frontier;
mod navigation;
mod plan;
mod policy;
mod ranges;
mod reserve_candidate;
mod reserve_evidence;
mod reserve_model;
mod reserve_origins;
mod reserve_schedule;
mod reserve_window;
mod reserves;
mod resources;
mod retrieval;
mod retrieval_ladder;
mod snapshot;
mod sources;
mod warp;

pub use allocation::REQUEST_SLICE_BYTES;
pub use catalog_snapshot::{candidate_snapshot, candidate_snapshot_at, CandidateEvidence};
pub use decision::{
    DecisionAction, DecisionModelInput, DecisionOutcome, DecisionPrivacy, DecisionRecord,
    DecisionRecordInput, DecisionReplayStatus, ExecutedRequest, ModelQuantiles, ProbeClaimRefusal,
    PrunedCandidate, PrunedReason, RecordedAllocationReason, RecordedCandidateUtility,
    RecordedExecutedRequest, RecordedHlsBootstrapStage, RecordedPlannerReplayCapsule,
    RecordedPlannerRetryAvailability, RecordedPlannerRetryEvidence, RecordedPreemptionAuthority,
    RecordedPromotionGrant, RecordedPrunedSearchPlan, RecordedRescueChanceEvidence,
    RecordedRescueTimingQuantile, RecordedReserveAuthorityOccupancy, RecordedReserveDegradedReason,
    RecordedResourceCost, RecordedResourcePrices, RecordedRetainedSearchPlan,
    RecordedRetrievalRequest, RecordedSearchPruneReason, RecordedTransfer, RecordedTransformKind,
    RecordedTwinEvaluation, RecordedWarpAction, RecordedWarpActionKind, RecordedWarpCommand,
    RecordedWarpDecision, RecordedWarpReserve, RecordedWarpSearch, RecordedWarpSearchInput,
    RecordedWholeBodyContract, RecordedWholeFetchReason, ShadowPrices, WarpDecisionRecordInput,
};
pub use navigation::{FeedOffset, NavigationDirection, NavigationHistory};
pub use plan::{
    Allocation, AllocationPlan, AllocationReason, CandidateUtility, ControlMode, DiscoveryDemand,
    Eviction, EvictionReason, NextReserveEvidence, NextReserveInfeasibility, PreemptionAuthority,
    ReadyReserveEvidence, ReserveCandidateEvidence, ReserveCandidateKind, ReserveCandidateState,
    RetainedAllocation,
};
pub use policy::AdaptivePlayabilityPolicy;
pub use retrieval::{
    PromotionGrant, PromotionOpportunity, RetrievalRequest, WholeBodyContract, WholeFetchReason,
};
pub use retrieval_ladder::{
    CompletionTimes, DeadlineReadiness, EpsilonBuckets, PlanMetrics, QualityEstimate,
    RetrievalLadder, RetrievalPlan, RetrievalRung, SizeBounds,
};
pub use snapshot::{
    CandidateSnapshot, CurrentAuthority, HlsBootstrapStage, HlsBootstrapState,
    HlsCandidateSnapshot, HlsObjectCursor, HlsTransport, InFlightAction, MediaLayout,
    NavigationSnapshot, NetworkSnapshot, OriginHealth, PlayabilitySnapshot, PlayableRange,
    PlaybackSnapshot, PlayerPreparation, SnapshotError, StorageSnapshot, ViewProbability,
};
pub use warp::{
    ActionForecast, ActionKind, ActionNode, ActionValue, ActiveControl, ActivePlannerContext,
    BeamConfig, CandidateRetrievalLadder, ContinuationDecision, DigitalTwin, GeneratedAction,
    GeneratedActions, HardBudget, HeadProbeHistory, HedgeInput, HedgePolicy, IdentityProof,
    NetworkTokenBucket, PlannerCandidateContext, PlannerCapability, PlannerCommand, PlannerContext,
    PlannerLimits, PlannerQuality, PlannerRetryAvailability, PlannerRetryEvidence,
    PlannerWatchEvidence, PreviewAvailability, PrunedSearchPlan, RequestOccupancy,
    RescueChanceEvidence, RescueTimingQuantile, ReserveAuthorityOccupancy, ReserveConstraint,
    ReserveDegradedReason, ResourceCost, ResourceFeedback, ResourceFeedbackCursor,
    ResourceObservation, ResourcePriceSnapshot, ResourcePrices, RetainedSearchPlan, SearchDecision,
    SearchPruneReason, SegmentedStorageBudget, SemanticAdmission, SemanticDecision, SemanticScore,
    ShadowPriceController, SoftRequestCommitment, TransformCapability, TransformKind,
    TransportCensorReason, TwinConfig, TwinEpochs, TwinEvaluation, TwinStateSignature, WarpPlanner,
    WarpPlannerConfig, WarpPlannerInput, WarpPlanningDecision, WholeBodyExhaustion,
};

pub(crate) use warp::{
    ContinuationPolicy, SemanticCandidate, SemanticGuardrail, TwinSearchContext, TwinState,
    WarpSearch,
};
pub(crate) use warp::{
    HlsGenerationPolicy, OriginAdmissionGenerationPolicy, PlannerReplayCapsule, PlannerReplayState,
    PromotionGenerationPolicy, RangeAliasGenerationPolicy, WarpGenerationPolicies,
};

#[cfg(test)]
#[path = "mod_axiom_test.rs"]
pub(crate) mod axiom_test_support;
