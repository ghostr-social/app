//! Bounded owners of active range-download tasks.

use crate::manager::inflight::{ChunkAttempt, CompletionStatus, InFlightChunks};
use crate::manager::plan::PlannedTransfer;
use crate::manager::transfers::{spawn_chunk, TransferContext};
use ghostr_engine::ChunkId;
use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct DownloadWorkers {
    active: InFlightChunks,
}

impl DownloadWorkers {
    pub fn new() -> Self {
        Self {
            active: InFlightChunks::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.active.len()
    }

    pub fn contains(&self, chunk: &ChunkId) -> bool {
        self.active.contains(chunk)
    }

    pub fn cancel_absent(&mut self, wanted: &HashSet<ChunkId>) {
        self.active.cancel_absent(wanted);
    }

    pub fn clear(&mut self) {
        self.active.cancel_absent(&HashSet::new());
    }

    pub fn start(&mut self, ctx: TransferContext, transfer: PlannedTransfer) {
        let chunk = transfer.request.chunk;
        let attempt = self.active.next_attempt(chunk);
        let handle = spawn_chunk(ctx, attempt.clone(), transfer.url);
        self.active.insert(&attempt, handle);
    }

    pub fn finish(&mut self, attempt: &ChunkAttempt) -> CompletionStatus {
        self.active.finish(attempt)
    }
}
