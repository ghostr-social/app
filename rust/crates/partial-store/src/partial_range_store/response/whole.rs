use super::{PartialRangeStore, ResponseOpenResult, StoreAction};
use crate::partial_range_store::single_response::{ResponseOwner, SingleResponseAuthority};
use anyhow::Result;
use ghostr_engine::adaptive::WholeBodyContract;
use ghostr_engine::representation::{HttpGenerationLease, TransferIdentity};

#[cfg(any(test, feature = "test"))]
mod test_support;

impl PartialRangeStore {
    /// # Errors
    ///
    /// Returns an error when generation authority or response state cannot be validated.
    pub async fn open_durable_single_response(
        &self,
        identity: &TransferIdentity,
        action: &StoreAction,
        contract: WholeBodyContract,
        generation: HttpGenerationLease,
    ) -> Result<ResponseOpenResult> {
        self.open_single_response(
            identity,
            action,
            contract,
            SingleResponseAuthority::Durable(generation),
        )
        .await
    }

    /// # Errors
    ///
    /// Returns an error when the action is stale or response state cannot be opened safely.
    pub async fn open_action_scoped_single_response(
        &self,
        identity: &TransferIdentity,
        action: &StoreAction,
        contract: WholeBodyContract,
    ) -> Result<ResponseOpenResult> {
        self.open_single_response(
            identity,
            action,
            contract,
            SingleResponseAuthority::ActionScoped,
        )
        .await
    }

    async fn open_single_response(
        &self,
        identity: &TransferIdentity,
        action: &StoreAction,
        contract: WholeBodyContract,
        authority: SingleResponseAuthority,
    ) -> Result<ResponseOpenResult> {
        if !action.is_active() || action.identity() != identity {
            return Ok(ResponseOpenResult::Stale);
        }
        let _update = self.update_key(identity.post().as_str()).await?;
        if !self
            .response_authority_is_current(identity, action, &authority)
            .await
        {
            return Ok(ResponseOpenResult::Stale);
        }
        self.open_single_response_action_locked(
            identity,
            ResponseOwner::Granted(action.clone()),
            contract,
            authority,
        )
        .await
    }

    async fn response_authority_is_current(
        &self,
        identity: &TransferIdentity,
        action: &StoreAction,
        authority: &SingleResponseAuthority,
    ) -> bool {
        if !action.is_active() || self.current_binding(identity).await.is_err() {
            return false;
        }
        match authority {
            SingleResponseAuthority::Durable(lease) => {
                self.http_generation_is_current(identity, lease).await
            }
            SingleResponseAuthority::ActionScoped => true,
            #[cfg(any(test, feature = "test"))]
            SingleResponseAuthority::Legacy => true,
        }
    }
}
