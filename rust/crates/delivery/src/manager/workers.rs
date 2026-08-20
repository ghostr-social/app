//! Bounded owners of active range-download tasks.

use crate::chunk::cancel::cancel_pair;
use crate::manager::admission::origin_key;
use crate::manager::inflight::{
    ActionRegistration, ActiveAction, ChunkAttempt, CompletionStatus, InFlightChunks,
};
use crate::manager::plan::PlannedTransfer;
use crate::manager::transfers::{spawn_chunk, TransferContext};
use ghostr_engine::representation::{RepresentationBinding, TransferIdentity};
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

    pub(crate) fn contains_transfer(&self, transfer: &PlannedTransfer) -> bool {
        self.active.contains_transfer(transfer)
    }

    pub(crate) fn actions(&self) -> Vec<ActiveAction> {
        self.active.actions()
    }

    pub(crate) fn body_posts(&self) -> HashSet<PostId> {
        self.active.body_posts()
    }

    pub(crate) fn contains_identity(&self, identity: &TransferIdentity) -> bool {
        self.active.contains_identity(identity)
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
        retained: &HashSet<ghostr_engine::ActionId>,
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

    pub async fn start(
        &mut self,
        ctx: TransferContext,
        transfer: PlannedTransfer,
    ) -> anyhow::Result<ghostr_engine::ActionId> {
        let host = origin_key(&transfer.url);
        let commitment_until_ms = transfer.commitment_until_ms;
        let retrieval = transfer.retrieval;
        let request = transfer.request;
        let chunk = request.chunk.clone();
        let attempt = self.active.next_attempt(chunk, transfer.identity);
        let action_id = attempt.id();
        let store_action = ctx
            .store
            .reserve_action(
                attempt.identity(),
                attempt.id().value(),
                retrieval.reserved_network_bytes(),
            )
            .await?;
        let (handle, token) = cancel_pair();
        self.active.insert_action(ActionRegistration {
            attempt: &attempt,
            priority: request,
            retrieval,
            host,
            committed_until_ms: commitment_until_ms,
            handle,
            store_action: Some(store_action.clone()),
        });
        spawn_chunk(super::transfers::ChunkLaunch {
            context: ctx,
            attempt,
            url: transfer.url,
            retrieval,
            token,
            action: store_action,
        });
        Ok(action_id)
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

    pub(crate) fn cancel_action(&mut self, action: ghostr_engine::ActionId) -> bool {
        self.active.cancel_action(action)
    }

    pub(crate) fn link_hedge(
        &mut self,
        primary: ghostr_engine::ActionId,
        alternate: ghostr_engine::ActionId,
    ) -> bool {
        self.active.link_hedge(primary, alternate)
    }

    pub(crate) fn complete_hedge_winner(&mut self, action: ghostr_engine::ActionId) -> bool {
        self.active.complete_hedge_winner(action)
    }

    pub(crate) fn observe_response(
        &mut self,
        attempt: &ChunkAttempt,
        response: crate::chunk::downloader::ResponseObservation,
    ) -> bool {
        self.active.observe_response(attempt, response)
    }

    pub(crate) fn authorizes_response(
        &self,
        attempt: &ChunkAttempt,
        action: &ghostr_partial_store::partial_range_store::StoreAction,
        response: &crate::chunk::downloader::OpenedResponse,
        opened_at_ms: u64,
    ) -> bool {
        self.active
            .authorizes_response(attempt, action, response, opened_at_ms)
    }

    pub(crate) fn reject_response(&mut self, attempt: &ChunkAttempt) {
        self.active.reject_response(attempt);
    }
}
