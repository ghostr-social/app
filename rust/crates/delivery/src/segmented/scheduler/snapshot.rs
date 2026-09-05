use super::SegmentedDelivery;
use crate::manager::state::DeliveryState;
use crate::segmented::SegmentedPhase;
use ghostr_engine::adaptive::{HlsBootstrapState, HlsCandidateSnapshot, NavigationSnapshot};

impl SegmentedDelivery {
    pub(crate) fn planning_candidates(
        &self,
        navigation: NavigationSnapshot,
        state: &DeliveryState,
    ) -> Vec<HlsCandidateSnapshot> {
        self.targets
            .iter()
            .map(|target| HlsCandidateSnapshot {
                post: target.post.clone(),
                feed_offset: target.offset,
                view_probability: navigation.view_probability(target.offset),
                startup_value_ms: self.startup_eta_ms,
                cursor: self.planning_cursor(&target.post),
                state: self.planning_state(&target.post),
                player_preparation: self.player_preparation(&target.post, state),
            })
            .collect()
    }

    pub(crate) fn available_bytes(&self) -> u64 {
        self.cache.planning_available_bytes()
    }

    pub(crate) fn used_bytes(&self) -> u64 {
        self.cache.physical_used_bytes()
    }

    pub(crate) fn active_sources(&self) -> Vec<&str> {
        self.active
            .values()
            .filter(|active| active.network.network_active())
            .map(|active| active.pending.url.as_str())
            .collect()
    }

    pub(crate) fn accepts_player_preparation(
        &self,
        report: &crate::delivery_events::PlayerPreparationReport,
    ) -> bool {
        report
            .hls_authority()
            .is_none_or(|authority| self.cache.accepts_prepared_authority(authority))
    }

    fn player_preparation(
        &self,
        post: &ghostr_engine::PostId,
        state: &DeliveryState,
    ) -> ghostr_engine::adaptive::PlayerPreparation {
        self.cache
            .snapshot(post.as_str())
            .authority
            .as_ref()
            .map_or_else(Default::default, |authority| {
                state.hls_player_preparation(authority)
            })
    }

    fn planning_state(&self, post: &ghostr_engine::PostId) -> HlsBootstrapState {
        let phase = self.cache.snapshot(post.as_str()).phase;
        if phase == SegmentedPhase::Failed {
            return HlsBootstrapState::Failed;
        }
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
        match phase {
            SegmentedPhase::Ready => HlsBootstrapState::Ready,
            SegmentedPhase::Failed | SegmentedPhase::Queued | SegmentedPhase::Preparing => {
                HlsBootstrapState::Failed
            }
        }
    }

    fn planning_cursor(
        &self,
        post: &ghostr_engine::PostId,
    ) -> ghostr_engine::adaptive::HlsObjectCursor {
        self.active
            .get(post)
            .map(|active| active.pending.cursor())
            .or_else(|| self.pending.get(post).map(super::progress::Pending::cursor))
            .unwrap_or_default()
    }
}
