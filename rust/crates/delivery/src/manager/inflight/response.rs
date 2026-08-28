use super::{InFlightChunks, ResponsePromotionStage};
use crate::chunk::downloader::{OpenedResponse, ResponseObservation};
use crate::manager::inflight::ChunkAttempt;
use ghostr_engine::adaptive::RetrievalRequest;
use ghostr_engine::ByteRange;
use ghostr_partial_store::partial_range_store::StoreAction;

mod transition;

impl InFlightChunks {
    pub(crate) fn observe_headers(
        &mut self,
        attempt: &ChunkAttempt,
        response: &OpenedResponse,
        observed_at_ms: u64,
    ) -> bool {
        let Some(active) = matching(self.transfers.get_mut(&attempt.id()), attempt) else {
            return false;
        };
        active.response_phase.stage(
            active.launched_request,
            attempt.profile().request(),
            response,
            observed_at_ms,
        );
        if matches!(response.observation(), ResponseObservation::Ignored { .. })
            || transition::covered(active.launched_request, response)
        {
            apply_response(active, response.observation());
        }
        true
    }

    pub(crate) fn stage_response_promotion(
        &mut self,
        attempt: &ChunkAttempt,
        action: &StoreAction,
        response: &OpenedResponse,
        observed_at_ms: u64,
    ) -> ResponsePromotionStage {
        let Some(active) = response_action(self.transfers.get_mut(&attempt.id()), attempt, action)
        else {
            return ResponsePromotionStage::Rejected;
        };
        active.response_phase.stage(
            active.launched_request,
            attempt.profile().request(),
            response,
            observed_at_ms,
        )
    }

    pub(crate) fn authorizes_response(
        &mut self,
        attempt: &ChunkAttempt,
        action: &StoreAction,
        response: &OpenedResponse,
        opened_at_ms: u64,
    ) -> bool {
        let Some(active) = response_action(self.transfers.get_mut(&attempt.id()), attempt, action)
        else {
            return false;
        };
        active.response_phase.stage(
            active.launched_request,
            attempt.profile().request(),
            response,
            opened_at_ms,
        );
        let allowed = transition::allowed(
            active.launched_request,
            active.promotion_authorization,
            active.response_phase,
            response,
            opened_at_ms,
        );
        if allowed {
            active.response_phase.open();
        }
        allowed
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
        let Some(active) = current(self.transfers.get_mut(&attempt.id()), attempt) else {
            return false;
        };
        active.response_phase.open();
        apply_response(active, response);
        true
    }
}

fn matching<'a>(
    active: Option<&'a mut super::action::ActiveChunk>,
    attempt: &ChunkAttempt,
) -> Option<&'a mut super::action::ActiveChunk> {
    active.filter(|active| active.identity == *attempt.identity())
}

fn current<'a>(
    active: Option<&'a mut super::action::ActiveChunk>,
    attempt: &ChunkAttempt,
) -> Option<&'a mut super::action::ActiveChunk> {
    matching(active, attempt).filter(|active| !active.cancelling)
}

fn response_action<'a>(
    active: Option<&'a mut super::action::ActiveChunk>,
    attempt: &ChunkAttempt,
    action: &StoreAction,
) -> Option<&'a mut super::action::ActiveChunk> {
    current(active, attempt).filter(|active| {
        active
            .store_action
            .as_ref()
            .is_some_and(|known| known.same_authority(action))
    })
}

fn apply_response(active: &mut super::action::ActiveChunk, response: ResponseObservation) {
    match response {
        ResponseObservation::Rejected(_) => {}
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
}
