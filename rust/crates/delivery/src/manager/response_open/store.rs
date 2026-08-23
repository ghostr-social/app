use super::{ChunkAttempt, OpenedResponse, ResponseOpenResult, StoreAction};
use crate::chunk::downloader::ResponseObservation;
use crate::manager::response_generation::ResponseAuthorityAdmission;
use crate::manager::DeliveryWorker;
use ghostr_engine::adaptive::WholeBodyContract;

impl DeliveryWorker {
    pub(super) async fn open_store_response(
        &self,
        attempt: &ChunkAttempt,
        action: &StoreAction,
        response: &OpenedResponse,
        admission: ResponseAuthorityAdmission,
    ) -> anyhow::Result<ResponseOpenResult> {
        match response.mode() {
            crate::chunk::sink::ResponseWriteMode::Sparse => {
                self.open_sparse_store_response(attempt, action, response)
                    .await
            }
            crate::chunk::sink::ResponseWriteMode::SingleResponse(contract) => {
                self.open_whole_store_response(attempt, action, contract, admission)
                    .await
            }
        }
    }

    async fn open_sparse_store_response(
        &self,
        attempt: &ChunkAttempt,
        action: &StoreAction,
        response: &OpenedResponse,
    ) -> anyhow::Result<ResponseOpenResult> {
        let Some(generation) = response.generation().cloned() else {
            return Ok(ResponseOpenResult::RequiresIndependentObject);
        };
        let ResponseObservation::Partial { range, .. } = response.observation() else {
            return Ok(ResponseOpenResult::RequiresIndependentObject);
        };
        self.ctx
            .store
            .open_sparse_response(attempt.identity(), action, generation, range)
            .await
    }

    async fn open_whole_store_response(
        &self,
        attempt: &ChunkAttempt,
        action: &StoreAction,
        contract: WholeBodyContract,
        admission: ResponseAuthorityAdmission,
    ) -> anyhow::Result<ResponseOpenResult> {
        match admission {
            ResponseAuthorityAdmission::Durable(generation) => {
                self.ctx
                    .store
                    .open_durable_single_response(attempt.identity(), action, contract, generation)
                    .await
            }
            ResponseAuthorityAdmission::ActionScoped => {
                self.ctx
                    .store
                    .open_action_scoped_single_response(attempt.identity(), action, contract)
                    .await
            }
        }
    }
}
