//! Bounded owners of active range-download tasks.

use ghostr_engine::ChunkId;
use crate::delivery_inflight::{ChunkAttempt, CompletionStatus, InFlightChunks};
use crate::delivery_plan::PlannedTransfer;
use crate::delivery_transfers::{spawn_chunk, TransferContext};
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
