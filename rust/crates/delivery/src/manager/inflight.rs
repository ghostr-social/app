//! Registry of in-flight chunk transfers and their cancel handles.
//! Handles must stay alive until cancelled: dropping one silently
//! would leave its transfer running unsupervised.

use crate::chunk::cancel::CancelHandle;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::scoring::ChunkRequest;
use ghostr_engine::tiers::Tier;
use ghostr_engine::ChunkId;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

mod reconciliation;

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
    request: ChunkRequest,
    started_as_seed: bool,
    io_finished: Arc<AtomicBool>,
    host: String,
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
        request: ChunkRequest,
        host: String,
        handle: CancelHandle,
    ) {
        let started_as_seed = request.tier == Tier::T2Startability;
        let active = ActiveChunk {
            id: attempt.id,
            identity: attempt.identity.clone(),
            request,
            started_as_seed,
            io_finished: Arc::clone(&attempt.io_finished),
            host,
            handle,
        };
        self.transfers.insert(attempt.chunk.clone(), active);
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
                        active.request.tier,
                        Tier::T0PlaybackEmergency | Tier::T1CurrentTail
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
}

fn overlaps(active: &ChunkId, request: &ChunkId) -> bool {
    active.post == request.post
        && active.range.start < request.range.end
        && request.range.start < active.range.end
}
