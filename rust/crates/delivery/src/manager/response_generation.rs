use crate::manager::inflight::ChunkAttempt;
use crate::manager::DeliveryWorker;
use ghostr_engine::representation::{HttpGenerationAuthority, TransferIdentity};

#[derive(Clone)]
pub(super) enum ResponseAuthorityAdmission {
    Durable(ghostr_engine::representation::HttpGenerationLease),
    ActionScoped,
}

impl DeliveryWorker {
    pub(super) async fn adopt_response_generation(
        &mut self,
        attempt: &ChunkAttempt,
        response: &crate::chunk::downloader::OpenedResponse,
    ) -> anyhow::Result<Option<ResponseAuthorityAdmission>> {
        if response.evidence().validator.is_none()
            || response.retention() == ghostr_net::media_retention::MediaRetention::Transient
            || self.has_other_continuation(attempt.identity()).await
        {
            self.learn_action_scoped_response(attempt, response, response.evidence().observed);
            let generation = self
                .state
                .catalog()
                .http_generation_stamp_for(attempt.identity());
            if !self
                .downloads
                .adopt_action_scoped_generation(attempt, generation)
            {
                return Ok(None);
            }
            return Ok(Some(ResponseAuthorityAdmission::ActionScoped));
        }
        let Some(generation) =
            self.learn_opened_response(attempt, response, response.evidence().observed)
        else {
            return Ok(None);
        };
        let authority = HttpGenerationAuthority::Trusted(generation.clone());
        if !self
            .install_http_authority(attempt.identity(), authority)
            .await?
        {
            return Ok(None);
        }
        Ok(self
            .downloads
            .adopt_http_generation(attempt, &generation)
            .then_some(ResponseAuthorityAdmission::Durable(generation)))
    }

    pub(super) async fn reject_response_generation(
        &mut self,
        attempt: &ChunkAttempt,
        response: &crate::chunk::downloader::OpenedResponse,
    ) -> anyhow::Result<bool> {
        let Some(authority) = self.reject_opened_generation(attempt, response) else {
            return Ok(false);
        };
        if !self.has_other_continuation(attempt.identity()).await
            && !self
                .install_http_authority(attempt.identity(), authority.clone())
                .await?
        {
            return Ok(false);
        }
        self.downloads
            .enforce_http_authority(attempt.identity(), &authority);
        Ok(true)
    }

    pub(super) async fn has_other_continuation(&self, identity: &TransferIdentity) -> bool {
        self.ctx
            .store
            .media_snapshot(identity.post().as_str())
            .await
            .is_ok_and(|snapshot| {
                snapshot
                    .continuation_source()
                    .is_some_and(|source| source != identity.source().as_str())
            })
    }

    async fn install_http_authority(
        &self,
        identity: &TransferIdentity,
        authority: HttpGenerationAuthority,
    ) -> anyhow::Result<bool> {
        self.ctx
            .store
            .apply_http_generation(identity, authority)
            .await
    }
}
