use super::{PartialRangeStore, ResponseOpenResult, StoreAction};
use crate::partial_range_store::single_response::{ResponseOwner, SingleResponseAuthority};
use anyhow::Result;
use ghostr_engine::adaptive::WholeBodyContract;
use ghostr_engine::representation::{HttpGenerationLease, TransferIdentity};

impl PartialRangeStore {
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
            SingleResponseAuthority::Legacy | SingleResponseAuthority::ActionScoped => true,
        }
    }
}
