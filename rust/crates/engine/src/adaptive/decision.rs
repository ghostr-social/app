//! Privacy-minimized decision records with deterministic replay.

mod advanced;
mod executed;
mod model;
mod plan;
mod plan_identity;
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
pub use types::{
    DecisionAction, DecisionModelInput, DecisionOutcome, DecisionReplayStatus, ModelQuantiles,
    ProbeClaimRefusal, PrunedCandidate, PrunedReason, ShadowPrices,
};

impl DecisionPrivacy {
    pub fn pseudonymized_post(&self, value: &str) -> String {
        self.post(value)
    }

    pub fn sanitized_plan(
        &self,
        value: &crate::adaptive::AllocationPlan,
    ) -> crate::adaptive::AllocationPlan {
        plan::sanitized(value, self)
    }
}
