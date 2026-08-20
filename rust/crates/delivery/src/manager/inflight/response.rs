use super::InFlightChunks;
use crate::chunk::downloader::{OpenedResponse, ResponseObservation, ResponseWriteMode};
use crate::manager::inflight::ChunkAttempt;
use ghostr_engine::adaptive::{RetrievalRequest, WholeBodyContract};
use ghostr_engine::ByteRange;
use ghostr_partial_store::partial_range_store::StoreAction;

impl InFlightChunks {
    pub(crate) fn authorizes_response(
        &self,
        attempt: &ChunkAttempt,
        action: &StoreAction,
        response: &OpenedResponse,
        opened_at_ms: u64,
    ) -> bool {
        let Some(active) = self.transfers.get(&attempt.id()) else {
            return false;
        };
        !active.cancelling
            && active.identity == *attempt.identity()
            && active
                .store_action
                .as_ref()
                .is_some_and(|known| known.same_authority(action))
            && transition_allowed(active.launched_request, response, opened_at_ms)
    }

    pub(crate) fn reject_response(&mut self, attempt: &ChunkAttempt) {
        if let Some(active) = self.transfers.get_mut(&attempt.id()) {
            active.cancel();
        }
    }

    pub(crate) fn observe_response(
        &mut self,
        attempt: &ChunkAttempt,
        response: ResponseObservation,
    ) -> bool {
        let Some(active) = self.transfers.get_mut(&attempt.id()) else {
            return false;
        };
        if active.cancelling || active.identity != *attempt.identity() {
            return false;
        }
        match response {
            ResponseObservation::Partial { range, .. } => {
                active.effective_request = RetrievalRequest::FetchRange {
                    bytes: range,
                    promotion: None,
                };
                active.effective_bytes = range;
                active.reserved_storage_bytes = range.len();
            }
            ResponseObservation::Body { request, .. } => {
                active.effective_request = request;
                active.effective_bytes = request.requested_bytes();
                active.reserved_storage_bytes = request.reserved_network_bytes();
            }
            ResponseObservation::Ignored { .. } => {
                let start = active.launched_request.requested_bytes().start;
                active.effective_bytes = ByteRange::new(start, start);
                active.reserved_storage_bytes = 0;
            }
        }
        true
    }
}

fn transition_allowed(
    launched: RetrievalRequest,
    response: &OpenedResponse,
    opened_at_ms: u64,
) -> bool {
    match (launched, response.observation(), response.mode()) {
        (
            RetrievalRequest::FetchRange { bytes, .. },
            ResponseObservation::Partial { range, .. },
            ResponseWriteMode::Sparse,
        ) => contains(bytes, range),
        (
            RetrievalRequest::FetchRange { bytes, .. },
            ResponseObservation::Body {
                request:
                    RetrievalRequest::FetchRange {
                        bytes: returned, ..
                    },
                ..
            },
            ResponseWriteMode::Sparse,
        ) => contains(bytes, returned),
        (RetrievalRequest::FetchRange { promotion, .. }, observation, mode) => {
            promoted_allowed(promotion, observation, mode, opened_at_ms)
        }
        (RetrievalRequest::FetchWhole { contract, reason }, observation, mode) => {
            whole_allowed(contract, reason, observation, mode)
        }
    }
}

fn promoted_allowed(
    grant: Option<ghostr_engine::adaptive::PromotionGrant>,
    observation: ResponseObservation,
    mode: ResponseWriteMode,
    opened_at_ms: u64,
) -> bool {
    let Some(grant) = grant.filter(|grant| opened_at_ms <= grant.valid_until_ms) else {
        return false;
    };
    let ResponseObservation::Body {
        request: RetrievalRequest::FetchWhole { contract, .. },
        promoted: true,
        ..
    } = observation
    else {
        return false;
    };
    mode == ResponseWriteMode::SingleResponse(contract)
        && contract.maximum_bytes() <= grant.maximum_bytes
}

fn whole_allowed(
    granted: WholeBodyContract,
    reason: ghostr_engine::adaptive::WholeFetchReason,
    observation: ResponseObservation,
    mode: ResponseWriteMode,
) -> bool {
    let ResponseObservation::Body {
        request:
            RetrievalRequest::FetchWhole {
                contract,
                reason: seen,
            },
        promoted: false,
        ..
    } = observation
    else {
        return false;
    };
    mode == ResponseWriteMode::SingleResponse(contract)
        && seen == reason
        && contract.maximum_bytes() <= granted.maximum_bytes()
}

fn contains(granted: ByteRange, returned: ByteRange) -> bool {
    granted.start <= returned.start && returned.end <= granted.end && !returned.is_empty()
}
