use crate::chunk::downloader::OpenedResponse;
use crate::manager::inflight::ChunkAttempt;
use crate::manager::DeliveryWorker;
use ghostr_partial_store::partial_range_store::StoreAction;

impl DeliveryWorker {
    pub(crate) async fn reject_unselected_response_for_test(
        &mut self,
        attempt: &ChunkAttempt,
        action: &StoreAction,
        response: &OpenedResponse,
    ) -> bool {
        self.response_identity_current(attempt, action, response)
            .await
    }
}
