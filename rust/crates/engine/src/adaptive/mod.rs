//! Pure adaptive allocation policy for progressive media delivery.

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
    DecisionRecordInput, DecisionReplayStatus, ModelQuantiles, PrunedCandidate, PrunedReason,
    ShadowPrices,
};
pub use navigation::{FeedOffset, NavigationDirection, NavigationHistory};
pub use plan::{
    Allocation, AllocationPlan, AllocationReason, CandidateUtility, ControlMode, DiscoveryDemand,
    Eviction, EvictionReason, NextReserveEvidence, NextReserveInfeasibility, PreemptionAuthority,
    ReadyReserveEvidence, ReserveCandidateEvidence, ReserveCandidateState, RetainedAllocation,
};
pub use policy::AdaptivePlayabilityPolicy;
pub use retrieval::{PromotionGrant, RetrievalRequest, WholeBodyContract, WholeFetchReason};
pub use retrieval_ladder::{
    CompletionTimes, DeadlineReadiness, EpsilonBuckets, PlanMetrics, QualityEstimate,
    RetrievalLadder, RetrievalPlan, RetrievalRung, SizeBounds,
};
pub use snapshot::{
    CandidateSnapshot, CurrentAuthority, InFlightAction, MediaLayout, NavigationSnapshot,
    NetworkSnapshot, OriginHealth, PlayabilitySnapshot, PlayableRange, PlaybackSnapshot,
    PlayerPreparation, SnapshotError, StorageSnapshot, ViewProbability,
};
pub use warp::{
    ActionForecast, ActionFrontier, ActionKind, ActionNode, ActionValue, ActiveControl,
    ActivePlannerContext, BeamConfig, CandidateRetrievalLadder, ContinuationDecision,
    ContinuationPolicy, DigitalTwin, GeneratedAction, GeneratedActions, HardBudget, HedgeInput,
    HedgePolicy, IdentityProof, NetworkTokenBucket, PlannerCandidateContext, PlannerCapability,
    PlannerCommand, PlannerContext, PlannerLimits, PlannerQuality, PreviewAvailability,
    PrunedSearchPlan, ReserveConstraint, ResourceCost, ResourceFeedback, ResourceObservation,
    ResourcePrices, RetainedSearchPlan, SearchDecision, SearchPruneReason, SemanticAdmission,
    SemanticCandidate, SemanticDecision, SemanticGuardrail, SemanticScore, ShadowPriceController,
    TransformCapability, TransformKind, TransportCensorReason, TwinConfig, TwinEpochs,
    TwinEvaluation, TwinSearchContext, TwinState, TwinStateSignature, WarpActionGenerator,
    WarpPlanner, WarpPlannerConfig, WarpPlannerInput, WarpPlanningDecision, WarpSearch,
};
