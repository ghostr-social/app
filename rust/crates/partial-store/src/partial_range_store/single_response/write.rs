use super::{PartialRangeStore, ResponseOwnerRef, SingleResponseState, SingleResponseStorage};
use crate::partial_range_store::StoreAction;
use anyhow::{ensure, Result};
use ghostr_engine::representation::TransferIdentity;

struct SingleResponseWrite<'a> {
    owner: ResponseOwnerRef<'a>,
    reservation: Option<&'a StoreAction>,
    offset: u64,
    bytes: &'a [u8],
}

#[cfg(any(test, feature = "test"))]
mod test_support;

impl PartialRangeStore {
    /// # Errors
    ///
    /// Returns an error when the action is stale or bytes cannot be persisted safely.
    pub async fn write_single_response_for_action(
        &self,
        identity: &TransferIdentity,
        action: &StoreAction,
        offset: u64,
        bytes: &[u8],
    ) -> Result<bool> {
        if !action.is_active() {
            return Ok(false);
        }
        let write = SingleResponseWrite {
            owner: ResponseOwnerRef::Granted(action),
            reservation: Some(action),
            offset,
            bytes,
        };
        self.write_single_response(identity, write).await
    }

    async fn write_single_response(
        &self,
        identity: &TransferIdentity,
        write: SingleResponseWrite<'_>,
    ) -> Result<bool> {
        let _update = self.update_key(identity.post().as_str()).await?;
        let Some(state) = self.current_single_response(identity, write.owner).await else {
            return Ok(false);
        };
        match state.storage {
            SingleResponseStorage::Memory => {
                self.write_transient(identity.post().as_str(), write.offset, write.bytes)
                    .await
            }
            SingleResponseStorage::Live { .. } => {
                self.write_live_single_response(identity, state, write)
                    .await
            }
            SingleResponseStorage::Staged { .. } => {
                self.write_staged_single_response(
                    identity,
                    state,
                    write.reservation,
                    write.offset,
                    write.bytes,
                )
                .await
            }
        }
    }

    async fn write_live_single_response(
        &self,
        identity: &TransferIdentity,
        state: SingleResponseState,
        write: SingleResponseWrite<'_>,
    ) -> Result<bool> {
        ensure!(!write.bytes.is_empty(), "single response write is empty");
        ensure!(
            write.offset.saturating_add(write.bytes.len() as u64) <= state.contract.maximum_bytes(),
            "response exceeds length"
        );
        let mut entries = self.entries.lock().await;
        match write.reservation {
            Some(action) => {
                self.write_range_locked_for_action(
                    &mut entries,
                    identity.post().as_str(),
                    action,
                    write.offset,
                    write.bytes,
                )
                .await?;
            }
            None => {
                self.write_range_locked(
                    &mut entries,
                    identity.post().as_str(),
                    write.offset,
                    write.bytes,
                )
                .await?;
            }
        }
        self.mark_live_started(identity.post().as_str(), state.owner.as_ref())
            .await;
        Ok(true)
    }

    async fn mark_live_started(&self, key: &str, owner: ResponseOwnerRef<'_>) {
        let mut actions = self.single_response_actions.lock().await;
        let Some(state) = actions
            .get_mut(key)
            .filter(|state| state.owner.matches(owner))
        else {
            return;
        };
        state.storage = SingleResponseStorage::Live { started: true };
    }
}
