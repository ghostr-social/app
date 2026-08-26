use super::evidence::FinalizedEvidence;
use crate::manager::DeliveryWorker;
use ghostr_engine::representation::TransferIdentity;
use ghostr_engine::PostId;

impl DeliveryWorker {
    pub(super) async fn transfer_is_current(&self, identity: &TransferIdentity) -> bool {
        if !self.ctx.store.transfer_is_current(identity).await {
            return false;
        }
        self.catalog_transfer_is_current(identity)
    }

    pub(crate) async fn transfer_binding_is_current(&self, identity: &TransferIdentity) -> bool {
        let stored = self
            .ctx
            .store
            .representation_binding(identity.post().as_str())
            .await;
        let matches = stored
            .and_then(|binding| binding.transfer(identity.source().as_str()))
            .as_ref()
            == Some(identity);
        matches && self.catalog_transfer_is_current(identity)
    }

    fn catalog_transfer_is_current(&self, identity: &TransferIdentity) -> bool {
        self.state
            .catalog()
            .transfer_identity(identity.post(), identity.source().as_str())
            .as_ref()
            == Some(identity)
    }

    pub(super) async fn try_finalize(
        &mut self,
        identity: &TransferIdentity,
        total: Option<u64>,
        response: Option<&crate::chunk::downloader::HttpResponseEvidence>,
        generation: Option<&ghostr_engine::representation::HttpGenerationLease>,
    ) {
        let post = identity.post();
        if !self.transfer_complete(post).await {
            return;
        }
        let advertised = self.advertised_digest(post);
        let outcome = self
            .ctx
            .store
            .finalize(post.as_str(), advertised.as_deref())
            .await;
        match outcome {
            Ok(completion) => self.learn_finalized(
                identity,
                &FinalizedEvidence {
                    total,
                    response,
                    advertised: advertised.as_deref(),
                    completion,
                    generation,
                },
            ),
            Err(error) => {
                self.finish_finalize_error(identity, advertised.as_deref(), error)
                    .await;
            }
        }
    }

    async fn transfer_complete(&self, post: &PostId) -> bool {
        self.ctx
            .store
            .is_complete(post.as_str())
            .await
            .unwrap_or(false)
    }

    fn advertised_digest(&self, post: &PostId) -> Option<String> {
        self.state
            .catalog()
            .lookup(post)
            .and_then(|entry| entry.meta.sha256.clone())
    }
}
