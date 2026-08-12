//! Bounded owners of active range-download tasks.

use crate::manager::admission::origin_key;
use crate::manager::inflight::{ActiveRange, ChunkAttempt, CompletionStatus, InFlightChunks};
use crate::manager::plan::PlannedTransfer;
use crate::manager::transfers::{spawn_chunk, TransferContext};
use ghostr_engine::representation::RepresentationBinding;
use ghostr_engine::{ChunkId, PostId};
use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct DownloadWorkers {
    active: InFlightChunks,
    admitted_capacity: usize,
}

impl DownloadWorkers {
    pub fn new() -> Self {
        Self {
            active: InFlightChunks::new(),
            admitted_capacity: 1,
        }
    }

    pub fn len(&self) -> usize {
        self.active.len()
    }

    pub fn contains(&self, chunk: &ChunkId) -> bool {
        self.active.contains(chunk)
    }

    pub(crate) fn ranges(&self) -> Vec<ActiveRange> {
        self.active.ranges()
    }

    pub fn preempt_for_current(&mut self, current: &PostId, priority: &[ChunkId], capacity: usize) {
        self.active.preempt_for_current(current, priority, capacity);
    }

    #[cfg(test)]
    pub fn reconcile(&mut self, planned: &[PlannedTransfer], capacity: usize) {
        self.reconcile_with_commitments(planned, capacity, &HashSet::new());
    }

    pub fn reconcile_with_commitments(
        &mut self,
        planned: &[PlannedTransfer],
        capacity: usize,
        retained: &HashSet<crate::manager::plan::PlannedTransferId>,
    ) {
        self.admitted_capacity = capacity.max(1);
        self.active
            .reconcile_with_commitments(planned, capacity, retained);
    }

    pub fn admitted_capacity(&self) -> usize {
        self.admitted_capacity.max(1)
    }

    pub fn clear(&mut self) {
        self.active.clear();
        self.admitted_capacity = 1;
    }

    pub(crate) fn cancel_obsolete(&mut self, binding: &RepresentationBinding) {
        self.active.cancel_obsolete(binding);
    }

    pub fn start(&mut self, ctx: TransferContext, transfer: PlannedTransfer) {
        let host = origin_key(&transfer.url);
        let commitment_until_ms = transfer.commitment_until_ms;
        let request = transfer.request;
        let chunk = request.chunk.clone();
        ctx.store.select_transfer(transfer.identity.clone());
        let attempt = self.active.next_attempt(chunk, transfer.identity);
        let handle = spawn_chunk(ctx, attempt.clone(), transfer.url);
        self.active
            .insert(&attempt, request, host, commitment_until_ms, handle);
    }

    pub fn active_hosts(&self) -> HashSet<String> {
        self.active.active_hosts()
    }

    pub fn foreground_len(&self) -> usize {
        self.active.foreground_len()
    }

    pub fn finish(&mut self, attempt: &ChunkAttempt) -> CompletionStatus {
        self.active.finish(attempt)
    }
}
