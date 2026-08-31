use super::DeliveryState;
use crate::delivery_events::PlayerPreparationReport;
use ghostr_engine::media_timeline::MediaTimeline;
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::PostId;
use ghostr_partial_store::partial_range_store::{ContentRevision, StoredMediaSnapshot};
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct FastStartEvidence {
    binding: RepresentationBinding,
    revision: ContentRevision,
}

impl FastStartEvidence {
    fn from_snapshot(
        binding: &RepresentationBinding,
        snapshot: &StoredMediaSnapshot,
        timeline: &MediaTimeline,
    ) -> Option<Self> {
        let total = snapshot.total_len()?;
        (snapshot.binding() == Some(binding)
            && snapshot.is_complete()
            && snapshot.is_finalized()
            && timeline.fast_start_remuxable(total))
        .then(|| Self {
            binding: binding.clone(),
            revision: snapshot.revision(),
        })
    }

    fn matches(&self, report: &PlayerPreparationReport, generation: Option<u64>) -> bool {
        report.progressive_binding() == Some(&self.binding)
            && report.progressive_revision() == Some(self.revision)
            && generation == Some(report.player_capability_generation())
    }
}

impl DeliveryState {
    pub(crate) fn reconcile_fast_start_evidence(
        &mut self,
        snapshots: &HashMap<PostId, StoredMediaSnapshot>,
    ) {
        self.fast_start_evidence = snapshots
            .iter()
            .filter_map(|(post, snapshot)| self.derive_fast_start(post, snapshot))
            .collect();
    }

    pub(super) fn current_fast_start_failure(&self, post: &PostId) -> bool {
        let Some(report) = self.player_preparations.get(post) else {
            return false;
        };
        self.fast_start_evidence.get(post).is_some_and(|evidence| {
            evidence.matches(report, self.client_capabilities.current_generation())
        })
    }

    fn derive_fast_start(
        &self,
        post: &PostId,
        snapshot: &StoredMediaSnapshot,
    ) -> Option<(PostId, FastStartEvidence)> {
        let binding = self.catalog.binding(post)?;
        let timeline = self.catalog.lookup(post)?.timeline()?;
        let evidence = FastStartEvidence::from_snapshot(&binding, snapshot, timeline)?;
        Some((post.clone(), evidence))
    }
}

#[cfg(test)]
mod tests;
