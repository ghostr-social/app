use super::{
    PartialRangeStore, ResponseOpenResult, SingleResponseAuthority, StoreAction, WholeBodyContract,
};
use anyhow::Result;
use ghostr_engine::representation::TransferIdentity;

impl PartialRangeStore {
    /// Opens a whole-body response for a legacy store action.
    ///
    /// # Errors
    ///
    /// Returns an error when response authority cannot be established.
    pub async fn open_single_response_for_action(
        &self,
        identity: &TransferIdentity,
        action: &StoreAction,
        contract: WholeBodyContract,
    ) -> Result<ResponseOpenResult> {
        let authority = self.http_generation_for(identity).await.map_or(
            SingleResponseAuthority::Legacy,
            SingleResponseAuthority::Durable,
        );
        self.open_single_response(identity, action, contract, authority)
            .await
    }
}
