use super::opening::SingleResponseOpening;
use super::{PartialRangeStore, ResponseOwner, SingleResponseAuthority, SingleResponseStorage};
use crate::partial_range_store::response::ResponseOpenResult;
use anyhow::{ensure, Result};
use ghostr_engine::adaptive::WholeBodyContract;
use ghostr_engine::representation::TransferIdentity;

impl PartialRangeStore {
    pub(in crate::partial_range_store) async fn open_single_response_action_locked(
        &self,
        identity: &TransferIdentity,
        owner: ResponseOwner,
        contract: WholeBodyContract,
        authority: SingleResponseAuthority,
    ) -> Result<ResponseOpenResult> {
        ensure!(
            contract.maximum_bytes() > 0,
            "single response limit must be positive"
        );
        let binding = self.current_binding(identity).await?;
        let key = identity.post().as_str();
        self.retry_inactive_single_response_locked(key).await?;
        if self.session_response(key).await.is_some() {
            return Ok(ResponseOpenResult::RequiresIndependentObject);
        }
        let opening = SingleResponseOpening::new(identity, owner, contract, authority);
        if let Some(result) = self.existing_single_response_open(key, &opening).await {
            return Ok(result);
        }
        let storage = self
            .single_response_storage(key, opening.contract(), opening.forces_staged_storage())
            .await?;
        if matches!(storage, SingleResponseStorage::Live { .. })
            && self.sparse_response_for_post(key).await
        {
            return Ok(ResponseOpenResult::RequiresIndependentObject);
        }
        self.prepare_live_exact_response(opening.identity(), &binding, opening.contract(), storage)
            .await?;
        self.single_response_actions
            .lock()
            .await
            .insert(key.to_owned(), opening.into_state(storage));
        Ok(ResponseOpenResult::Opened)
    }

    async fn existing_single_response_open(
        &self,
        key: &str,
        opening: &SingleResponseOpening<'_>,
    ) -> Option<ResponseOpenResult> {
        self.single_response_actions
            .lock()
            .await
            .get(key)
            .map(|known| {
                if opening.matches(known) {
                    ResponseOpenResult::Opened
                } else {
                    ResponseOpenResult::RequiresIndependentObject
                }
            })
    }
}
