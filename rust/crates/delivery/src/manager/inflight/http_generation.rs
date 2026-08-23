use super::{ChunkAttempt, InFlightChunks};
use ghostr_engine::representation::{
    HttpGenerationAuthority, HttpGenerationLease, HttpGenerationStamp, TransferIdentity,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ResponseGenerationFence {
    Durable(HttpGenerationLease),
    ActionScoped(Option<HttpGenerationStamp>),
}

impl InFlightChunks {
    pub(crate) fn adopt_http_generation(
        &mut self,
        attempt: &ChunkAttempt,
        generation: HttpGenerationLease,
    ) -> bool {
        let Some(active) = self.transfers.get_mut(&attempt.id()) else {
            return false;
        };
        if active.cancelling || active.identity != *attempt.identity() {
            return false;
        }
        active.http_generation = Some(generation.clone());
        active.response_generation_fence =
            Some(ResponseGenerationFence::Durable(generation.clone()));
        let identity = active.identity.clone();
        for (id, candidate) in &mut self.transfers {
            if *id != attempt.id()
                && candidate.identity == identity
                && candidate.http_generation.as_ref() != Some(&generation)
            {
                candidate.cancel();
            }
        }
        true
    }

    pub(crate) fn enforce_http_authority(
        &mut self,
        identity: &TransferIdentity,
        authority: &HttpGenerationAuthority,
    ) {
        for active in self.transfers.values_mut() {
            let accepted = match authority {
                HttpGenerationAuthority::Trusted(lease) => {
                    active.http_generation.as_ref() == Some(lease)
                }
                HttpGenerationAuthority::Unknown(_) => false,
            };
            if active.identity == *identity && !accepted {
                active.cancel();
            }
        }
    }

    pub(crate) fn http_generation(&self, attempt: &ChunkAttempt) -> Option<HttpGenerationLease> {
        let active = self.transfers.get(&attempt.id())?;
        (active.identity == *attempt.identity())
            .then(|| active.http_generation.clone())
            .flatten()
    }

    pub(crate) fn adopt_action_scoped_generation(
        &mut self,
        attempt: &ChunkAttempt,
        generation: Option<HttpGenerationStamp>,
    ) -> bool {
        let Some(active) = self.transfers.get_mut(&attempt.id()) else {
            return false;
        };
        if active.cancelling || active.identity != *attempt.identity() {
            return false;
        }
        active.response_generation_fence = Some(ResponseGenerationFence::ActionScoped(generation));
        true
    }

    pub(crate) fn policy_limit_generation(
        &self,
        attempt: &ChunkAttempt,
    ) -> Option<ResponseGenerationFence> {
        let active = self.transfers.get(&attempt.id())?;
        (active.identity == *attempt.identity() && active.chunk == attempt.chunk)
            .then(|| active.response_generation_fence.clone())
            .flatten()
    }
}
