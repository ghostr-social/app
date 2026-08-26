use super::*;
use crate::adaptive::{PlannerCandidateContext, TwinEpochs};
use crate::origin_model::NetworkClass;

impl WarpPlanningDecision {
    pub fn planner_candidate_evidence(&self, post: &PostId) -> Option<PlannerCandidateContext> {
        self.planner_replay.as_ref()?.context().candidate(post)
    }

    pub fn planner_epochs(&self) -> Option<TwinEpochs> {
        Some(self.planner_replay.as_ref()?.context().epochs)
    }

    pub fn planner_network_class(&self) -> Option<NetworkClass> {
        Some(self.planner_replay.as_ref()?.context().network_class())
    }
}
