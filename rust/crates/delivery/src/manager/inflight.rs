//! Registry of in-flight chunk transfers and their cancel handles.
//! Handles must stay alive until cancelled: dropping one silently
//! would leave its transfer running unsupervised.

use crate::chunk::cancel::CancelHandle;
use ghostr_engine::ChunkId;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct InFlightChunks {
    transfers: HashMap<ChunkId, ActiveChunk>,
    next_id: u64,
}

#[derive(Clone, Debug)]
pub(crate) struct ChunkAttempt {
    pub chunk: ChunkId,
    id: u64,
}

struct ActiveChunk {
    id: u64,
    handle: CancelHandle,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CompletionStatus {
    Current,
    Untracked,
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
        self.transfers.contains_key(chunk)
    }

    pub fn next_attempt(&mut self, chunk: ChunkId) -> ChunkAttempt {
        self.next_id = self
            .next_id
            .checked_add(1)
            .expect("chunk attempt id exhausted");
        ChunkAttempt {
            chunk,
            id: self.next_id,
        }
    }

    pub fn insert(&mut self, attempt: &ChunkAttempt, handle: CancelHandle) {
        let active = ActiveChunk {
            id: attempt.id,
            handle,
        };
        self.transfers.insert(attempt.chunk.clone(), active);
    }

    pub fn finish(&mut self, attempt: &ChunkAttempt) -> CompletionStatus {
        let Some(active) = self.transfers.get(&attempt.chunk) else {
            return CompletionStatus::Untracked;
        };
        if active.id != attempt.id {
            return CompletionStatus::Superseded;
        }
        self.transfers.remove(&attempt.chunk);
        CompletionStatus::Current
    }

    /// Cancels every transfer the fresh plan no longer wants (the
    /// scroll-past rule) and frees their slots immediately; fetched
    /// bytes stay in the store, resumable.
    pub fn cancel_absent(&mut self, wanted: &HashSet<ChunkId>) {
        self.transfers.retain(|chunk, active| {
            let keep = wanted.contains(chunk);
            if !keep {
                active.handle.cancel();
            }
            keep
        });
    }
}
