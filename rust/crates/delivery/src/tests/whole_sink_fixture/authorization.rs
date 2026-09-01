use crate::chunk::downloader::{OpenedResponse, ResponseAdmission, ResponseObservation};
use crate::chunk::sink::ResponseWriteMode;
use crate::chunk::traffic::ChunkTraffic;
use core::future::Future;
use core::pin::Pin;
use core::time::Duration;
use ghostr_engine::representation::TransferIdentity;
use ghostr_partial_store::partial_range_store::{
    PartialRangeStore, ResponseOpenResult, StoreAction,
};
use std::sync::Arc;

pub(crate) struct AuthorizedTraffic {
    store: Arc<PartialRangeStore>,
    identity: TransferIdentity,
    action: StoreAction,
}

impl AuthorizedTraffic {
    pub(crate) fn new(
        store: Arc<PartialRangeStore>,
        identity: TransferIdentity,
        action: StoreAction,
    ) -> Self {
        Self {
            store,
            identity,
            action,
        }
    }
}

impl ChunkTraffic for AuthorizedTraffic {
    fn opened(&mut self, _: Duration) {}

    fn wrote(&mut self, _: u64) {}

    fn authorize_response<'a>(
        &'a mut self,
        response: OpenedResponse,
    ) -> Pin<Box<dyn Future<Output = anyhow::Result<ResponseAdmission>> + Send + 'a>> {
        Box::pin(async move {
            let result = match response.mode() {
                ResponseWriteMode::Sparse => self.open_sparse(&response).await?,
                ResponseWriteMode::SingleResponse(contract) => {
                    self.store
                        .open_single_response_for_action(&self.identity, &self.action, contract)
                        .await?
                }
            };
            Ok(if result == ResponseOpenResult::Opened {
                ResponseAdmission::Proceed
            } else {
                ResponseAdmission::Reject
            })
        })
    }
}

impl AuthorizedTraffic {
    async fn open_sparse(&self, response: &OpenedResponse) -> anyhow::Result<ResponseOpenResult> {
        let Some(generation) = response.generation().cloned() else {
            return Ok(ResponseOpenResult::Stale);
        };
        let ResponseObservation::Partial { range, .. } = response.observation() else {
            return Ok(ResponseOpenResult::Stale);
        };
        self.store
            .open_sparse_response(&self.identity, &self.action, generation, range)
            .await
    }
}
