//! Bounded owners of active range-download tasks.

use crate::manager::admission::origin_key;
use crate::manager::inflight::{ChunkAttempt, CompletionStatus, InFlightChunks};
use crate::manager::plan::{PlannedTransfer, PlannedTransferId};
use crate::manager::transfers::{spawn_chunk, TransferContext};
use ghostr_engine::{ChunkId, PostId};
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

    pub fn preempt_for_current(&mut self, current: &PostId, priority: &[ChunkId], capacity: usize) {
        self.active.preempt_for_current(current, priority, capacity);
    }

    pub fn cancel_absent(&mut self, wanted: &HashSet<PlannedTransferId>) {
        self.active.cancel_absent(wanted);
    }

    pub fn clear(&mut self) {
        self.active.clear();
    }

    pub fn start(&mut self, ctx: TransferContext, transfer: PlannedTransfer) {
        let host = origin_key(&transfer.url);
        let chunk = transfer.request.chunk;
        ctx.store.select_transfer(transfer.identity.clone());
        let attempt = self.active.next_attempt(chunk, transfer.identity);
        let handle = spawn_chunk(ctx, attempt.clone(), transfer.url);
        self.active.insert(&attempt, host, handle);
    }

    pub fn active_hosts(&self) -> HashSet<String> {
        self.active.active_hosts()
    }

    pub fn finish(&mut self, attempt: &ChunkAttempt) -> CompletionStatus {
        self.active.finish(attempt)
    }
}
