//! Renewable rate leases for an already-open continuous response.
use crate::manager::inflight::ChunkAttempt;
use crate::manager::{time, DeliveryWorker};
use tokio::sync::oneshot;

pub(crate) struct BodyRenewalRequest {
    pub(crate) attempt: ChunkAttempt,
    pub(crate) through: u64,
    pub(crate) reply: oneshot::Sender<BodyRenewal>,
}

pub(crate) enum BodyRenewal {
    Granted,
    WaitUntil(u64),
    Denied,
}

impl DeliveryWorker {
    pub(super) fn renew_body(&mut self, request: BodyRenewalRequest) {
        if request.reply.is_closed() { return; }
        let result = self.body_renewal(&request);
        let _ = request.reply.send(result);
    }

    fn body_renewal(&mut self, request: &BodyRenewalRequest) -> BodyRenewal {
        let identity = request.attempt.identity();
        if self.state.catalog().transfer_identity(identity.post(), identity.source().as_str()).as_ref() != Some(identity) {
            return BodyRenewal::Denied;
        }
        let Some(delta) = self.downloads.body_renewal_delta(&request.attempt, request.through) else {
            return BodyRenewal::Denied;
        };
        if delta == 0 { return BodyRenewal::Granted; }
        let now = time::unix_time_ms();
        if self.warp_planner.reserve_network_window(delta, now) {
            self.downloads.commit_body_renewal(&request.attempt, delta);
            return BodyRenewal::Granted;
        }
        self.warp_planner.network_window_deadline_ms(delta, now)
            .map_or(BodyRenewal::Denied, BodyRenewal::WaitUntil)
    }
}
