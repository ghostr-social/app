//! Bounded owners of active range-download tasks.

use crate::manager::inflight::{ActiveAction, ChunkAttempt, CompletionStatus, InFlightChunks};
use crate::manager::plan::PlannedTransfer;
use ghostr_engine::representation::{RepresentationBinding, TransferIdentity};
use ghostr_engine::{ChunkId, PostId};
use std::collections::HashSet;

mod start;
pub(crate) use start::PreparedTransfer;

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

    pub(crate) fn next_action_id(&mut self) -> ghostr_engine::ActionId {
        self.active.next_action_id()
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

    pub(crate) fn can_cancel_action(&self, action: ghostr_engine::ActionId) -> bool {
        self.active.can_cancel_action(action)
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
        &mut self,
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

    pub(crate) fn preflight_promotion(
        &self,
        target: &crate::manager::inflight::PromotionTarget,
        now_ms: u64,
    ) -> Result<
        crate::manager::inflight::PromotionPreflight,
        crate::manager::inflight::PromotionRejection,
    > {
        self.active.preflight_promotion(target, now_ms)
    }

    pub(crate) fn activate_promotion(
        &mut self,
        preflight: &crate::manager::inflight::PromotionPreflight,
        now_ms: u64,
    ) -> bool {
        self.active.activate_promotion(preflight, now_ms)
    }

    pub(crate) fn rollback_promotion(
        &mut self,
        preflight: &crate::manager::inflight::PromotionPreflight,
    ) -> bool {
        self.active.rollback_promotion(preflight)
    }
}
