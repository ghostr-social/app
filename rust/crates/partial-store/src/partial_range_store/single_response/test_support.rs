use super::{PartialRangeStore, ResponseOwner, SingleResponseAuthority, WholeBodyContract};
use crate::partial_range_store::response::ResponseOpenResult;
#[cfg(test)]
use crate::partial_range_store::StoreAction;
use anyhow::{ensure, Result};
use ghostr_engine::representation::TransferIdentity;

impl PartialRangeStore {
    #[cfg(test)]
    pub(crate) async fn begin_single_response_for_action(
        &self,
        identity: &TransferIdentity,
        action: &StoreAction,
        contract: WholeBodyContract,
    ) -> Result<bool> {
        Ok(matches!(
            self.open_single_response_for_action(identity, action, contract)
                .await?,
            ResponseOpenResult::Opened
        ))
    }

    /// Begins a response owned by a legacy test action.
    ///
    /// # Errors
    ///
    /// Returns an error for an invalid contract or unavailable store state.
    pub async fn begin_single_response(
        &self,
        identity: &TransferIdentity,
        action: u64,
        contract: WholeBodyContract,
    ) -> Result<bool> {
        ensure!(
            contract.maximum_bytes() > 0,
            "single response limit must be positive"
        );
        let _update = self.update_key(identity.post().as_str()).await?;
        self.current_binding(identity).await?;
        if self.selected().get(identity.post().as_str()) != Some(identity) {
            return Ok(false);
        }
        Ok(matches!(
            self.open_single_response_action_locked(
                identity,
                ResponseOwner::Legacy(action),
                contract,
                SingleResponseAuthority::Legacy,
            )
            .await?,
            ResponseOpenResult::Opened
        ))
    }
}
