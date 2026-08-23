use super::{ChunkAttempt, DownloadWorkers, TransferIdentity};
use crate::manager::inflight::ResponseGenerationFence;
use ghostr_engine::representation::{
    HttpGenerationAuthority, HttpGenerationLease, HttpGenerationStamp,
};

impl DownloadWorkers {
    pub(crate) fn adopt_http_generation(
        &mut self,
        attempt: &ChunkAttempt,
        generation: HttpGenerationLease,
    ) -> bool {
        self.active.adopt_http_generation(attempt, generation)
    }

    pub(crate) fn enforce_http_authority(
        &mut self,
        identity: &TransferIdentity,
        authority: &HttpGenerationAuthority,
    ) {
        self.active.enforce_http_authority(identity, authority);
    }

    pub(crate) fn http_generation(&self, attempt: &ChunkAttempt) -> Option<HttpGenerationLease> {
        self.active.http_generation(attempt)
    }

    pub(crate) fn adopt_action_scoped_generation(
        &mut self,
        attempt: &ChunkAttempt,
        generation: Option<HttpGenerationStamp>,
    ) -> bool {
        self.active
            .adopt_action_scoped_generation(attempt, generation)
    }

    pub(crate) fn policy_limit_generation(
        &self,
        attempt: &ChunkAttempt,
    ) -> Option<ResponseGenerationFence> {
        self.active.policy_limit_generation(attempt)
    }
}
