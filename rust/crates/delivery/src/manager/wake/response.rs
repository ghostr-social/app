use super::DeliveryWorker;
use crate::manager::inflight::ResponsePromotionStage;
use crate::manager::response_open::ResponseOpenRequest;

impl DeliveryWorker {
    pub(super) async fn step_response(&mut self, request: ResponseOpenRequest) {
        if self.stage_response_open(&request) == ResponsePromotionStage::NewOpportunity {
            self.apply_pending_focus().await;
            Box::pin(self.reconcile()).await;
            self.apply_response_open(request).await;
            return;
        }
        self.apply_response_open(request).await;
        self.apply_pending_focus().await;
        Box::pin(self.reconcile()).await;
    }
}
