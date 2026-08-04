//! Registry of in-flight chunk transfers and their cancel handles.
//! Handles must stay alive until cancelled: dropping one silently
//! would leave its transfer running unsupervised.

use crate::engine::ChunkId;
use crate::video::chunk_cancel::CancelHandle;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct InFlightChunks {
    transfers: HashMap<ChunkId, CancelHandle>,
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

    pub fn insert(&mut self, chunk: ChunkId, handle: CancelHandle) {
        self.transfers.insert(chunk, handle);
    }

    pub fn remove(&mut self, chunk: &ChunkId) {
        self.transfers.remove(chunk);
    }

    /// Cancels every transfer the fresh plan no longer wants (the
    /// scroll-past rule) and frees their slots immediately; fetched
    /// bytes stay in the store, resumable.
    pub fn cancel_absent(&mut self, wanted: &HashSet<ChunkId>) {
        self.transfers.retain(|chunk, handle| {
            let keep = wanted.contains(chunk);
            if !keep {
                handle.cancel();
            }
            keep
        });
    }
}
