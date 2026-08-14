//! Registry of in-flight chunk transfers and their cancel handles.
//! Handles must stay alive until cancelled: dropping one silently
//! would leave its transfer running unsupervised.

use crate::chunk::cancel::CancelHandle;
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_engine::representation::{RepresentationBinding, TransferIdentity};
use ghostr_engine::scheduling::RangeRequest;
use ghostr_engine::{ChunkId, PostId};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod reconciliation;
mod snapshot;

pub(crate) use snapshot::ActiveRange;

#[derive(Default)]
pub(crate) struct InFlightChunks {
    transfers: HashMap<ChunkId, ActiveChunk>,
    next_id: u64,
}

#[derive(Clone, Debug)]
pub(crate) struct ChunkAttempt {
    pub chunk: ChunkId,
    identity: TransferIdentity,
    id: u64,
    io_finished: Arc<AtomicBool>,
}

impl ChunkAttempt {
    pub(crate) fn id(&self) -> u64 {
        self.id
    }

    pub(crate) fn identity(&self) -> &TransferIdentity {
        &self.identity
    }

    pub(crate) fn mark_io_finished(&self) {
        self.io_finished.store(true, Ordering::Release);
    }
}

struct ActiveChunk {
    id: u64,
    identity: TransferIdentity,
    request: RangeRequest,
    policy_retained: bool,
    io_finished: Arc<AtomicBool>,
    host: String,
    committed_until_ms: u64,
    handle: CancelHandle,
}

impl ActiveChunk {
    fn io_finished(&self) -> bool {
        self.io_finished.load(Ordering::Acquire)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CompletionStatus {
    Current,
    Superseded,
}

impl InFlightChunks {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.transfers
            .values()
            .filter(|active| !active.io_finished())
            .count()
    }

    pub fn contains(&self, chunk: &ChunkId) -> bool {
        self.transfers.iter().any(|(active, state)| {
            overlaps(active, chunk) || (state.io_finished() && active.post == chunk.post)
        })
    }

    pub fn cancel(&mut self, chunk: &ChunkId) -> bool {
        let Some(active) = self.transfers.remove(chunk) else {
            return false;
        };
        active.handle.cancel();
        true
    }

    pub fn next_attempt(&mut self, chunk: ChunkId, identity: TransferIdentity) -> ChunkAttempt {
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("chunk attempt id exhausted");
        ChunkAttempt {
            chunk,
            identity,
            id: self.next_id,
            io_finished: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn insert(
        &mut self,
        attempt: &ChunkAttempt,
        request: RangeRequest,
        host: String,
        committed_until_ms: u64,
        handle: CancelHandle,
    ) {
        let active = ActiveChunk {
            id: attempt.id,
            identity: attempt.identity.clone(),
            request,
            policy_retained: false,
            io_finished: Arc::clone(&attempt.io_finished),
            host,
            committed_until_ms,
            handle,
        };
        self.transfers.insert(attempt.chunk.clone(), active);
    }

    pub(crate) fn body_posts(&self) -> HashSet<PostId> {
        self.transfers
            .keys()
            .map(|chunk| chunk.post.clone())
            .collect()
    }

    pub fn active_hosts(&self) -> HashSet<String> {
        self.transfers
            .values()
            .filter(|active| !active.io_finished())
            .map(|active| active.host.clone())
            .collect()
    }

    pub fn foreground_len(&self) -> usize {
        self.transfers
            .values()
            .filter(|active| {
                !active.io_finished()
                    && matches!(
                        active.request.authority,
                        PreemptionAuthority::PlaybackCritical
                    )
            })
            .count()
    }

    pub fn finish(&mut self, attempt: &ChunkAttempt) -> CompletionStatus {
        let Some(active) = self.transfers.get(&attempt.chunk) else {
            return CompletionStatus::Superseded;
        };
        if active.id != attempt.id || active.identity != attempt.identity {
            return CompletionStatus::Superseded;
        }
        self.transfers.remove(&attempt.chunk);
        CompletionStatus::Current
    }

    pub fn clear(&mut self) {
        for active in self.transfers.values() {
            active.handle.cancel();
        }
        self.transfers.clear();
    }

    pub(crate) fn cancel_obsolete(&mut self, binding: &RepresentationBinding) {
        self.transfers.retain(|chunk, active| {
            let current = binding.transfer(active.identity.source().as_str());
            let keep = chunk.post != *binding.post() || current.as_ref() == Some(&active.identity);
            if !keep {
                active.handle.cancel();
            }
            keep
        });
    }
}

fn overlaps(active: &ChunkId, request: &ChunkId) -> bool {
    active.post == request.post
        && active.range.start < request.range.end
        && request.range.start < active.range.end
}
