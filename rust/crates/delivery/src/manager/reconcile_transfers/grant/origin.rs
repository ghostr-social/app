use super::admitted::AdmittedGrant;
use super::disposition::GrantRejection;
use crate::manager::plan::PlannedTransfer;
use crate::manager::{origin_admission, time, DeliveryWorker};
use ghostr_engine::origin_model::ClaimedAdmission;
use ghostr_engine::RequestAuthority;

#[cfg(test)]
#[path = "origin_concurrency_test.rs"]
mod origin_concurrency_test;

impl DeliveryWorker {
    pub(super) fn admit_origin(
        &mut self,
        transfer: PlannedTransfer,
    ) -> Result<AdmittedGrant, GrantRejection> {
        let observed_at_ms = time::unix_time_ms();
        let authority =
            RequestAuthority::from_url(&transfer.url).ok_or(GrantRejection::InvalidAuthority)?;
        let claimed = self.claim_origin(&transfer, &authority, observed_at_ms);
        let reason = claimed.block_reason();
        let (admission, claim) = claimed.into_parts();
        if let Some(reason) = reason {
            return Err(GrantRejection::Origin(reason));
        }
        let transfer = origin_admission::apply(transfer, &admission)
            .ok_or(GrantRejection::AdmissionInvariant)?;
        Ok(AdmittedGrant::new(transfer, observed_at_ms, claim))
    }

    fn claim_origin(
        &mut self,
        transfer: &PlannedTransfer,
        authority: &RequestAuthority,
        observed_at_ms: u64,
    ) -> ClaimedAdmission {
        let concurrency = origin_concurrency(&self.ctx.requests, authority);
        let query = origin_admission::query(
            transfer,
            observed_at_ms,
            concurrency,
            self.state.network_class(),
        );
        self.keeper.stats_mut().origin_model_mut().claim(
            &query,
            observed_at_ms,
            origin_admission::mode(transfer),
            transfer.profile.admission_intent(),
        )
    }
}

fn origin_concurrency(
    requests: &ghostr_net::media_request_executor::MediaRequestExecutor,
    authority: &RequestAuthority,
) -> usize {
    requests
        .active_for(authority)
        .saturating_add(1)
        .min(requests.limits().per_authority())
}
