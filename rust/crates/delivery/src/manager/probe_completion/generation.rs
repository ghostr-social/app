use crate::manager::DeliveryWorker;
use crate::probe::media::ProbeResult;
use ghostr_engine::catalog::{HttpObservation, LearnedFacts};
use ghostr_engine::host_stats::host_of;
use ghostr_engine::representation::{HttpGenerationStamp, TransferIdentity};

impl DeliveryWorker {
    pub(in crate::manager) async fn absorb_probe(
        &mut self,
        identity: &TransferIdentity,
        result: &ProbeResult,
    ) -> anyhow::Result<Option<HttpGenerationStamp>> {
        self.note_successful_attempt(identity.post(), identity.source().as_str());
        let observation = HttpObservation::new(
            facts(result),
            result.content_type.clone(),
            result.observed,
            result.validator.clone(),
        )
        .with_final_url(result.final_url.clone())
        .with_request_selection(result.request_selection);
        let stamp = self
            .state
            .catalog_mut()
            .learn_head_observation_with_stamp_for(identity, observation);
        let Some(stamp) = stamp else { return Ok(None) };
        self.install_probe_generation(identity, &stamp).await?;
        Ok(Some(stamp))
    }

    async fn install_probe_generation(
        &mut self,
        identity: &TransferIdentity,
        stamp: &HttpGenerationStamp,
    ) -> anyhow::Result<()> {
        if self.has_other_continuation(identity).await {
            return Ok(());
        }
        let authority = stamp.authority().clone();
        if self
            .ctx
            .store
            .apply_http_generation(identity, authority.clone())
            .await?
        {
            self.downloads.enforce_http_authority(identity, &authority);
        }
        Ok(())
    }
}

fn facts(result: &ProbeResult) -> LearnedFacts {
    LearnedFacts {
        content_length: result.content_length,
        accept_ranges: result.accept_ranges,
        host: host_of(&result.final_url),
    }
}
