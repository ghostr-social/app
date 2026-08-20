use crate::chunk::downloader::ResponseAdmission;
use crate::evaluation::IntegrityMetricEvent;
use crate::manager::DeliveryWorker;

impl DeliveryWorker {
    pub(super) fn reject_stale_response(&self) -> ResponseAdmission {
        let metrics = self.commands.evaluation();
        metrics.integrity(IntegrityMetricEvent::StaleValidator);
        metrics.integrity(IntegrityMetricEvent::IncorrectRangeSplicePrevented);
        ResponseAdmission::Reject
    }
}
