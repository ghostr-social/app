use super::{ChunkAttempt, OpenedResponse, StoreAction};
use crate::manager::DeliveryWorker;

impl DeliveryWorker {
    pub(in crate::manager) async fn response_identity_current(
        &mut self,
        attempt: &ChunkAttempt,
        action: &StoreAction,
        response: &OpenedResponse,
    ) -> bool {
        let observed_at_ms = response.evidence().observed.observed_at_ms;
        if !self
            .downloads
            .authorizes_response(attempt, action, response, observed_at_ms)
        {
            self.downloads.reject_response(attempt);
            return false;
        }
        if self.transfer_binding_is_current(attempt.identity()).await {
            return true;
        }
        self.downloads.reject_response(attempt);
        false
    }
}
