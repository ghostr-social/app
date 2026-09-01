use super::lifecycle::AttemptLifecycle;
use crate::chunk::cancel::CancelHandle;
use ghostr_engine::adaptive::RetrievalRequest;
use ghostr_engine::origin_model::{AdmissionClaim, OriginAttemptProfile};
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::scheduling::RangeRequest;
use ghostr_engine::{ActionId, ChunkId};
use ghostr_partial_store::partial_range_store::StoreAction;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(crate) struct ChunkAttempt {
    pub chunk: ChunkId,
    identity: TransferIdentity,
    id: ActionId,
    profile: OriginAttemptProfile,
    lifecycle: Arc<AttemptLifecycle>,
}

pub(crate) struct ActionRegistration<'a> {
    pub(crate) attempt: &'a ChunkAttempt,
    pub(crate) priority: RangeRequest,
    pub(crate) retrieval: RetrievalRequest,
    pub(crate) host: String,
    pub(crate) committed_until_ms: u64,
    pub(crate) launched_at_ms: u64,
    pub(crate) handle: CancelHandle,
    pub(crate) store_action: Option<StoreAction>,
    pub(crate) committed_network_bytes: Option<u64>,
    pub(crate) admission_claim: Option<AdmissionClaim>,
}

impl ChunkAttempt {
    pub(super) fn new_with_profile(
        chunk: ChunkId,
        identity: TransferIdentity,
        id: ActionId,
        profile: OriginAttemptProfile,
    ) -> Self {
        Self {
            chunk,
            identity,
            id,
            profile,
            lifecycle: Arc::new(AttemptLifecycle::default()),
        }
    }

    pub(crate) fn id(&self) -> ActionId {
        self.id
    }

    pub(crate) fn identity(&self) -> &TransferIdentity {
        &self.identity
    }

    pub(crate) const fn profile(&self) -> OriginAttemptProfile {
        self.profile
    }

    pub(crate) fn mark_io_finished(&self) {
        self.lifecycle.mark_io_finished();
    }
}

pub(super) struct ActiveChunk {
    pub(super) action_id: ActionId,
    pub(super) chunk: ChunkId,
    pub(super) identity: TransferIdentity,
    pub(super) priority: RangeRequest,
    pub(super) launched_request: RetrievalRequest,
    pub(super) effective_request: RetrievalRequest,
    pub(super) effective_bytes: ghostr_engine::ByteRange,
    pub(super) reserved_storage_bytes: u64,
    pub(super) committed_network_bytes: u64,
    pub(super) uncommitted_network_prefix_bytes: u64,
    pub(super) policy_retained: bool,
    lifecycle: Arc<AttemptLifecycle>,
    pub(super) host: String,
    pub(super) committed_until_ms: u64,
    pub(super) launched_at_ms: u64,
    handle: CancelHandle,
    pub(super) store_action: Option<StoreAction>,
    pub(super) promotion_authorization: Option<ghostr_engine::adaptive::PromotionGrant>,
    pub(super) http_generation: Option<ghostr_engine::representation::HttpGenerationLease>,
    pub(super) response_generation_fence: Option<super::ResponseGenerationFence>,
    pub(super) response_phase: super::ResponsePhase,
    pub(super) hedge_disposition: Option<HedgeDisposition>,
    pub(super) cancelling: bool,
    pub(super) admission_claim: Option<AdmissionClaim>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum HedgeDisposition {
    Winner,
    Loser,
}

impl ActiveChunk {
    pub(super) fn from_registration(registration: ActionRegistration<'_>) -> Self {
        let effective_bytes = registration.retrieval.requested_bytes();
        let reserved_storage_bytes = registration.retrieval.immediate_network_bytes();
        let committed_network_bytes = registration.committed_network_bytes.unwrap_or(0);
        let uncommitted_network_prefix_bytes = registration
            .committed_network_bytes
            .map_or(reserved_storage_bytes, |_| 0);
        Self {
            action_id: registration.attempt.id,
            chunk: registration.attempt.chunk.clone(),
            identity: registration.attempt.identity.clone(),
            priority: registration.priority,
            launched_request: registration.retrieval,
            effective_request: registration.retrieval,
            effective_bytes,
            reserved_storage_bytes,
            committed_network_bytes,
            uncommitted_network_prefix_bytes,
            policy_retained: false,
            lifecycle: Arc::clone(&registration.attempt.lifecycle),
            host: registration.host,
            committed_until_ms: registration.committed_until_ms,
            launched_at_ms: registration.launched_at_ms,
            handle: registration.handle,
            store_action: registration.store_action,
            promotion_authorization: None,
            http_generation: None,
            response_generation_fence: None,
            response_phase: super::ResponsePhase::AwaitingHeaders,
            hedge_disposition: None,
            cancelling: false,
            admission_claim: registration.admission_claim,
        }
    }

    pub(super) fn io_finished(&self) -> bool {
        self.lifecycle.io_finished()
    }

    pub(super) fn authorize_hedge(&self) -> bool {
        self.lifecycle.authorize_hedge()
    }

    pub(super) fn hedge_authorized(&self) -> bool {
        self.lifecycle.hedge_authorized()
    }

    pub(super) fn release_hedge(&self) {
        self.lifecycle.release_hedge();
    }

    pub(super) fn cancel(&mut self) -> bool {
        if self.cancelling || !self.lifecycle.begin_cancel() {
            return false;
        }
        self.cancelling = true;
        if let Some(action) = &self.store_action {
            action.revoke();
        }
        self.handle.cancel();
        true
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CompletionStatus {
    Current,
    Cancelled,
    HedgeWinner,
    HedgeLoser,
    Superseded,
}

#[cfg(test)]
#[path = "action_axiom_test.rs"]
mod axiom_test_support;
