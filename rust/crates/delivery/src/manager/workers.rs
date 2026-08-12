//! Bounded owners of active range-download tasks.

use crate::manager::admission::origin_key;
use crate::manager::inflight::{ChunkAttempt, CompletionStatus, InFlightChunks};
use crate::manager::plan::eviction::ProtectedSeedEviction;
use crate::manager::plan::PlannedTransfer;
use crate::manager::transfers::{spawn_chunk, TransferContext};
use ghostr_engine::representation::TransferIdentity;
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

    pub fn preempt_for_current(
        &mut self,
        current: &PostId,
        priority: &[ChunkId],
        capacity: usize,
        eviction: ProtectedSeedEviction,
    ) {
        self.active
            .preempt_for_current_with_eviction(current, priority, capacity, eviction);
    }

    #[cfg(test)]
    pub fn reconcile(&mut self, planned: &[PlannedTransfer], capacity: usize) {
        self.reconcile_with_commitments(
            planned,
            capacity,
            ProtectedSeedEviction::Allow,
            &HashSet::new(),
        );
    }

    pub fn reconcile_with_commitments(
        &mut self,
        planned: &[PlannedTransfer],
        capacity: usize,
        eviction: ProtectedSeedEviction,
        protected_identities: &HashSet<TransferIdentity>,
    ) {
        self.admitted_capacity = capacity.max(1);
        self.active
            .reconcile_with_commitments(planned, capacity, eviction, protected_identities);
    }

    pub fn admitted_capacity(&self) -> usize {
        self.admitted_capacity.max(1)
    }

    pub fn clear(&mut self) {
        self.active.clear();
        self.admitted_capacity = 1;
    }

    pub fn start(&mut self, ctx: TransferContext, transfer: PlannedTransfer) {
        let host = origin_key(&transfer.url);
        let request = transfer.request;
        let chunk = request.chunk.clone();
        ctx.store.select_transfer(transfer.identity.clone());
        let attempt = self.active.next_attempt(chunk, transfer.identity);
        let handle = spawn_chunk(ctx, attempt.clone(), transfer.url);
        self.active.insert(&attempt, request, host, handle);
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
