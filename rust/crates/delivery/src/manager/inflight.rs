//! Registry of in-flight chunk transfers and their cancel handles.
//! Handles must stay alive until cancelled: dropping one silently
//! would leave its transfer running unsupervised.

use crate::chunk::cancel::CancelHandle;
use crate::manager::plan::PlannedTransferId;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::{ChunkId, PostId};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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
    io_finished: Arc<AtomicBool>,
    host: String,
    handle: CancelHandle,
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
        self.transfers.len()
    }

    pub fn contains(&self, chunk: &ChunkId) -> bool {
        self.transfers.keys().any(|active| covers(active, chunk))
    }

    pub fn cancel(&mut self, chunk: &ChunkId) -> bool {
        let Some(active) = self.transfers.remove(chunk) else {
            return false;
        };
        active.handle.cancel();
        true
    }

    pub fn preempt_for_current(&mut self, current: &PostId, priority: &[ChunkId], capacity: usize) {
        if capacity == 0 || self.transfers.keys().any(|chunk| &chunk.post == current) {
            return;
        }
        let Some(rank) = priority.iter().position(|chunk| &chunk.post == current) else {
            return;
        };
        while self.len() >= capacity {
            let Some(victim) = self.lower_priority_victim(current, &priority[rank + 1..]) else {
                return;
            };
            self.cancel(&victim);
        }
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

    pub fn insert(&mut self, attempt: &ChunkAttempt, host: String, handle: CancelHandle) {
        let active = ActiveChunk {
            id: attempt.id,
            identity: attempt.identity.clone(),
            io_finished: Arc::clone(&attempt.io_finished),
            host,
            handle,
        };
        self.transfers.insert(attempt.chunk.clone(), active);
    }

    pub fn active_hosts(&self) -> HashSet<String> {
        self.transfers
            .values()
            .map(|active| active.host.clone())
            .collect()
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

    /// Cancels every transfer the fresh plan no longer wants (the
    /// scroll-past rule) and frees their slots immediately; fetched
    /// bytes stay in the store, resumable.
    pub fn cancel_absent(&mut self, wanted: &HashSet<PlannedTransferId>) {
        self.transfers.retain(|chunk, active| {
            let keep = active.io_finished.load(Ordering::Acquire)
                || wanted
                    .iter()
                    .any(|request| covers_identity(chunk, &active.identity, request));
            if !keep {
                active.handle.cancel();
            }
            keep
        });
    }

    pub fn clear(&mut self) {
        for active in self.transfers.values() {
            active.handle.cancel();
        }
        self.transfers.clear();
    }

    fn lower_priority_victim(&self, current: &PostId, priority: &[ChunkId]) -> Option<ChunkId> {
        priority.iter().rev().find_map(|request| {
            self.transfers
                .keys()
                .find(|active| &active.post != current && covers(active, request))
                .cloned()
        })
    }
}

fn covers(active: &ChunkId, request: &ChunkId) -> bool {
    active.post == request.post
        && active.range.start <= request.range.start
        && active.range.end >= request.range.end
}

fn covers_identity(
    active: &ChunkId,
    identity: &TransferIdentity,
    request: &PlannedTransferId,
) -> bool {
    identity == &request.identity && covers(active, &request.chunk)
}
