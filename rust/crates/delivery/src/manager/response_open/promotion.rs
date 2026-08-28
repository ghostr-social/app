use super::ResponseOpenRequest;
use crate::manager::inflight::ResponsePromotionStage;
use crate::manager::DeliveryWorker;

impl DeliveryWorker {
    pub(in crate::manager) fn stage_response_open(
        &mut self,
        request: &ResponseOpenRequest,
    ) -> ResponsePromotionStage {
        if request.reply.is_closed() || request.opened_at.elapsed() > self.ctx.timeouts.idle {
            self.downloads.reject_response(&request.attempt);
            return ResponsePromotionStage::Rejected;
        }
        self.downloads.stage_response_promotion(
            &request.attempt,
            &request.action,
            &request.response,
            crate::manager::time::unix_time_ms(),
        )
    }
}
