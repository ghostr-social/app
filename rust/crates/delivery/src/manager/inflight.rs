//! Registry of in-flight chunk transfers and their cancel handles.
//! Handles must stay alive until cancelled: dropping one silently
//! would leave its transfer running unsupervised.

use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::{ActionId, ChunkId, PostId};
use std::collections::{BTreeMap, HashSet};

mod action;
mod cancellation;
mod hedge;
mod http_generation;
mod network_reservation;
mod promotion;
mod reconciliation;
mod response;
mod response_phase;
mod snapshot;

use action::ActiveChunk;
pub(crate) use action::{ActionRegistration, ChunkAttempt, CompletionStatus};
pub(crate) use http_generation::ResponseGenerationFence;
pub(crate) use network_reservation::FinishedAction;
pub(crate) use promotion::{PromotionPreflight, PromotionRejection, PromotionTarget};
use response_phase::ResponsePhase;
pub(crate) use response_phase::ResponsePromotionStage;
pub(crate) use snapshot::ActiveAction;

#[derive(Default)]
pub(crate) struct InFlightChunks {
    transfers: BTreeMap<ActionId, ActiveChunk>,
    hedges: BTreeMap<ActionId, ActionId>,
    next_id: u64,
}

impl InFlightChunks {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.transfers.len()
    }

    pub fn contains(&self, chunk: &ChunkId) -> bool {
        self.transfers.values().any(|active| {
            overlaps(&active.chunk, chunk)
                || (active.io_finished() && active.chunk.post == chunk.post)
        })
    }

    pub fn next_attempt_with_profile(
        &mut self,
        chunk: ChunkId,
        identity: TransferIdentity,
        profile: ghostr_engine::origin_model::OriginAttemptProfile,
    ) -> ChunkAttempt {
        ChunkAttempt::new_with_profile(chunk, identity, self.next_action_id(), profile)
    }

    #[cfg(test)]
    pub fn next_attempt(&mut self, chunk: ChunkId, identity: TransferIdentity) -> ChunkAttempt {
        let profile = action::test_profile(chunk.range.len());
        self.next_attempt_with_profile(chunk, identity, profile)
    }

    pub(super) fn next_action_id(&mut self) -> ActionId {
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("chunk attempt id exhausted");
        ActionId::new(self.next_id)
    }

    pub fn insert_action(&mut self, registration: ActionRegistration<'_>) {
        let active = ActiveChunk::from_registration(registration);
        self.transfers.insert(active.action_id, active);
    }

    pub(crate) fn body_posts(&self) -> HashSet<PostId> {
        self.transfers
            .values()
            .map(|active| active.chunk.post.clone())
            .collect()
    }

    pub fn active_hosts(&self) -> HashSet<String> {
        self.transfers
            .values()
            .map(|active| active.host.clone())
            .collect()
    }

    pub fn foreground_len(&self) -> usize {
        self.transfers
            .values()
            .filter(|active| {
                !active.cancelling
                    && !active.io_finished()
                    && matches!(
                        active.priority.authority,
                        PreemptionAuthority::PlaybackCritical
                    )
            })
            .count()
    }

    pub fn clear(&mut self) {
        for active in self.transfers.values_mut() {
            active.cancel();
        }
        self.hedges.clear();
    }
}

fn overlaps(active: &ChunkId, request: &ChunkId) -> bool {
    active.post == request.post
        && active.range.start < request.range.end
        && request.range.start < active.range.end
}

#[cfg(test)]
#[path = "inflight_axiom_test.rs"]
pub(crate) mod axiom_test_support;
