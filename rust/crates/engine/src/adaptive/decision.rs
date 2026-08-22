//! Privacy-minimized decision records with deterministic replay.

mod advanced;
mod executed;
mod model;
mod plan;
mod privacy;
mod record;
mod replay;
mod state;
mod types;

pub use advanced::{
    RecordedAllocationReason, RecordedCandidateUtility, RecordedExecutedRequest,
    RecordedHlsBootstrapStage, RecordedPlannerReplayCapsule, RecordedPlannerRetryAvailability,
    RecordedPlannerRetryEvidence, RecordedPreemptionAuthority, RecordedPromotionGrant,
    RecordedPrunedSearchPlan, RecordedRescueChanceEvidence, RecordedRescueTimingQuantile,
    RecordedReserveAuthorityOccupancy, RecordedReserveDegradedReason, RecordedResourceCost,
    RecordedResourcePrices, RecordedRetainedSearchPlan, RecordedRetrievalRequest,
    RecordedSearchPruneReason, RecordedTransfer, RecordedTransformKind, RecordedTwinEvaluation,
    RecordedWarpAction, RecordedWarpActionKind, RecordedWarpCommand, RecordedWarpDecision,
    RecordedWarpReserve, RecordedWarpSearch, RecordedWarpSearchInput, RecordedWholeBodyContract,
    RecordedWholeFetchReason,
};
pub use executed::ExecutedRequest;
pub use privacy::DecisionPrivacy;
pub use record::{DecisionRecord, DecisionRecordInput, WarpDecisionRecordInput};
pub use replay::{VerifiedWarpReplay, WarpReplayIntegrity};
pub use types::{
    DecisionAction, DecisionModelInput, DecisionOutcome, DecisionReplayStatus, ModelQuantiles,
    ProbeClaimRefusal, PrunedCandidate, PrunedReason, ShadowPrices,
};
