use super::PlannerContext;
use crate::adaptive::{PlannerRetryAvailability, PlannerRetryEvidence, PlayabilitySnapshot};
use std::collections::HashSet;

impl PlannerContext {
    pub(in crate::adaptive::warp) fn retry_evidence(
        &self,
        snapshot: &PlayabilitySnapshot,
    ) -> Vec<PlannerRetryEvidence> {
        let mut seen = HashSet::new();
        snapshot
            .candidates
            .iter()
            .map(|item| &item.post)
            .chain(snapshot.hls_candidates.iter().map(|item| &item.post))
            .filter(|post| seen.insert((*post).clone()))
            .filter_map(|post| self.evidence(post))
            .collect()
    }

    fn evidence(&self, post: &crate::PostId) -> Option<PlannerRetryEvidence> {
        let retry = self.candidates.get(post)?.retry;
        (retry != PlannerRetryAvailability::Ready)
            .then(|| PlannerRetryEvidence::new(post.clone(), retry))
    }
}
