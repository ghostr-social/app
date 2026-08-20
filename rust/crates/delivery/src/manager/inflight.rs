//! Registry of in-flight chunk transfers and their cancel handles.
//! Handles must stay alive until cancelled: dropping one silently
//! would leave its transfer running unsupervised.

use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::representation::{RepresentationBinding, TransferIdentity};
use ghostr_engine::{ActionId, ChunkId, PostId};
use std::collections::{HashMap, HashSet};

mod action;
mod reconciliation;
mod response;
mod snapshot;

use action::ActiveChunk;
pub(crate) use action::{ActionRegistration, ChunkAttempt, CompletionStatus};
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

    #[cfg(test)]
    pub fn cancel(&mut self, chunk: &ChunkId) -> bool {
        let Some(active) = self
            .transfers
            .values_mut()
            .find(|active| overlaps(&active.chunk, chunk) && !active.cancelling)
        else {
            return false;
        };
        active.cancel();
        true
    }

    pub(crate) fn cancel_action(&mut self, action: ActionId) -> bool {
        let Some(active) = self.transfers.get_mut(&action) else {
            return false;
        };
        active.cancel();
        true
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
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("chunk attempt id exhausted");
        ChunkAttempt::new(chunk, identity, ActionId::new(self.next_id))
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

    pub(crate) fn cancel_obsolete(&mut self, binding: &RepresentationBinding) {
        for active in self.transfers.values_mut() {
            let current = binding.transfer(active.identity.source().as_str());
            let obsolete =
                active.chunk.post == *binding.post() && current.as_ref() != Some(&active.identity);
            if obsolete {
                active.cancel();
            }
        }
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
