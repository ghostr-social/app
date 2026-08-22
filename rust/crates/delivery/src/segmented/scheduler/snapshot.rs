use super::SegmentedDelivery;
use crate::segmented::SegmentedPhase;
use ghostr_engine::adaptive::{HlsBootstrapState, HlsCandidateSnapshot, NavigationSnapshot};

impl SegmentedDelivery {
    pub(crate) fn planning_candidates(
        &self,
        navigation: NavigationSnapshot,
    ) -> Vec<HlsCandidateSnapshot> {
        self.targets
            .iter()
            .map(|target| HlsCandidateSnapshot {
                post: target.post.clone(),
                feed_offset: target.offset,
                view_probability: navigation.view_probability(target.offset),
                startup_value_ms: self.startup_eta_ms,
                state: self.planning_state(&target.post),
            })
            .collect()
    }

    pub(crate) fn available_bytes(&self) -> u64 {
        self.cache.planning_available_bytes()
    }

    pub(crate) fn active_sources(&self) -> Vec<&str> {
        self.active
            .values()
            .map(|active| active.pending.url.as_str())
            .collect()
    }

    fn planning_state(&self, post: &ghostr_engine::PostId) -> HlsBootstrapState {
        if let Some(active) = self.active.get(post) {
            return HlsBootstrapState::Active {
                action: active.action,
                stage: active.pending.stage,
                source: active.pending.url.clone(),
                committed_until_ms: active.committed_until_ms,
                cancelling: active.cancelling,
            };
        }
        if let Some(pending) = self.pending.get(post) {
            return HlsBootstrapState::Pending {
                stage: pending.stage,
                source: pending.url.clone(),
            };
        }
        match self.cache.snapshot(post.as_str()).phase {
            SegmentedPhase::Ready => HlsBootstrapState::Ready,
            SegmentedPhase::Failed => HlsBootstrapState::Failed,
            SegmentedPhase::Queued | SegmentedPhase::Preparing => HlsBootstrapState::Failed,
        }
    }
}
