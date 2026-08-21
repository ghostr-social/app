//! Registry of in-flight chunk transfers and their cancel handles.
//! Handles must stay alive until cancelled: dropping one silently
//! would leave its transfer running unsupervised.

use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::{ActionId, ChunkId, PostId};
use std::collections::{HashMap, HashSet};

mod action;
mod cancellation;
mod promotion;
mod reconciliation;
mod response;
mod snapshot;

use action::ActiveChunk;
pub(crate) use action::{ActionRegistration, ChunkAttempt, CompletionStatus};
pub(crate) use promotion::{PromotionPreflight, PromotionRejection, PromotionTarget};
pub(crate) use snapshot::ActiveAction;

#[derive(Default)]
pub(crate) struct InFlightChunks {
    transfers: HashMap<ActionId, ActiveChunk>,
    hedges: HashMap<ActionId, ActionId>,
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

    pub(crate) fn link_hedge(&mut self, primary: ActionId, alternate: ActionId) -> bool {
        if primary == alternate
            || !self.transfers.contains_key(&primary)
            || !self.transfers.contains_key(&alternate)
        {
            return false;
        }
        self.hedges.insert(primary, alternate);
        self.hedges.insert(alternate, primary);
        true
    }

    pub(crate) fn complete_hedge_winner(&mut self, winner: ActionId) -> bool {
        let Some(loser) = self.hedges.get(&winner).copied() else {
            return false;
        };
        self.cancel_action(loser)
    }

    pub fn next_attempt(&mut self, chunk: ChunkId, identity: TransferIdentity) -> ChunkAttempt {
        ChunkAttempt::new(chunk, identity, self.next_action_id())
    }

    pub(crate) fn next_action_id(&mut self) -> ActionId {
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("chunk attempt id exhausted");
        ActionId::new(self.next_id)
    }

    #[cfg(test)]
    pub fn insert(
        &mut self,
        attempt: &ChunkAttempt,
        priority: ghostr_engine::scheduling::RangeRequest,
        host: String,
        committed_until_ms: u64,
        handle: crate::chunk::cancel::CancelHandle,
    ) {
        let retrieval = ghostr_engine::adaptive::RetrievalRequest::FetchRange {
            bytes: priority.chunk.range,
            promotion: None,
        };
        self.insert_action(ActionRegistration {
            attempt,
            priority,
            retrieval,
            host,
            committed_until_ms,
            launched_at_ms: 0,
            handle,
            store_action: None,
        });
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

    pub fn finish(&mut self, attempt: &ChunkAttempt) -> CompletionStatus {
        let Some(active) = self.transfers.get(&attempt.id()) else {
            return CompletionStatus::Superseded;
        };
        if active.identity != *attempt.identity() || active.chunk != attempt.chunk {
            return CompletionStatus::Superseded;
        }
        let cancelled = active.cancelling;
        self.transfers.remove(&attempt.id());
        self.unlink_hedge(attempt.id());
        match cancelled {
            true => CompletionStatus::Cancelled,
            false => CompletionStatus::Current,
        }
    }

    pub fn clear(&mut self) {
        for active in self.transfers.values_mut() {
            active.cancel();
        }
        self.transfers.clear();
        self.hedges.clear();
    }

    fn unlink_hedge(&mut self, action: ActionId) {
        if let Some(peer) = self.hedges.remove(&action) {
            self.hedges.remove(&peer);
        }
    }
}

fn overlaps(active: &ChunkId, request: &ChunkId) -> bool {
    active.post == request.post
        && active.range.start < request.range.end
        && request.range.start < active.range.end
}
