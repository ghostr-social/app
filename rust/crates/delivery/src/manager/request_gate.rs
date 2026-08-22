use crate::manager::concurrency::RequestConcurrencyLimits;
use crate::manager::DeliveryWorker;
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};

pub(super) fn apply_request_limits(
    requests: &MediaRequestExecutor,
    limits: RequestConcurrencyLimits,
) {
    let limits = MediaRequestLimits::try_new(limits.global(), limits.per_authority())
        .expect("resolved request limits are nonzero and ordered");
    requests.update_limits(limits);
}

impl DeliveryWorker {
    pub(super) fn sync_request_gate(&self, limits: RequestConcurrencyLimits) {
        apply_request_limits(&self.ctx.requests, limits);
    }
}
