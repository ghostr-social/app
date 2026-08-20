use crate::partial_range_disk as disk;
use crate::partial_range_store::{PartialRangeStore, StoreAction};
use anyhow::{ensure, Result};
use ghostr_engine::representation::{SourceGeneration, TransferIdentity};
use ghostr_engine::ByteRange;
use sha2::Sha256;

#[derive(Clone)]
pub(in crate::partial_range_store) struct SparseResponseState {
    pub(in crate::partial_range_store) owner: StoreAction,
    pub(in crate::partial_range_store) identity: TransferIdentity,
    pub(in crate::partial_range_store) generation: SourceGeneration,
    pub(in crate::partial_range_store) range: ByteRange,
    pub(in crate::partial_range_store) next_offset: u64,
    pub(in crate::partial_range_store) received: u64,
    pub(in crate::partial_range_store) pending: u64,
    pub(in crate::partial_range_store) hasher: Sha256,
    pub(in crate::partial_range_store) intent_installed: bool,
    pub(in crate::partial_range_store) dirty: bool,
    pub(in crate::partial_range_store) committed: bool,
}

impl PartialRangeStore {
    pub async fn write_range_for_action_if_current(
        &self,
        identity: &TransferIdentity,
        generation: &SourceGeneration,
        action: &StoreAction,
        offset: u64,
        bytes: &[u8],
    ) -> Result<bool> {
        let _update = self.update_key(identity.post().as_str()).await?;
        let Some(state) = self.sparse_state(identity, generation, action).await else {
            return Ok(false);
        };
        if !self.generation_is_current(identity, generation).await {
            return Ok(false);
        }
        validate_write(&state, action, offset, bytes)?;
        if bytes.is_empty() {
            return Ok(true);
        }
        self.install_sparse_intent(&state).await?;
        self.write_sparse_bytes(identity, action, offset, bytes)
            .await?;
        Ok(action.is_active()
            && self
                .sparse_action_is_current(identity, generation, action)
                .await)
    }

    async fn write_sparse_bytes(
        &self,
        identity: &TransferIdentity,
        action: &StoreAction,
        offset: u64,
        bytes: &[u8],
    ) -> Result<()> {
        let _capacity = self.capacity_updates.lock().await;
        if self.shortfall(0).await > 0 {
            let mut entries = self.entries.lock().await;
            self.make_room(&mut entries, identity.post().as_str(), 0)
                .await?;
        }
        self.charge_action_write(action, bytes.len() as u64).await?;
        self.mark_sparse_attempt(action, bytes.len() as u64).await?;
        disk::write_at_unsynced(&self.paths.partial(identity.post().as_str()), offset, bytes)
            .await?;
        self.record_sparse_write(action, bytes).await?;
        self.changed.notify_waiters();
        Ok(())
    }

    pub async fn finish_sparse_response(
        &self,
        identity: &TransferIdentity,
        generation: &SourceGeneration,
        action: &StoreAction,
    ) -> Result<bool> {
        let key = identity.post().as_str();
        let _update = self.update_key(key).await?;
        let Some(state) = self.sparse_state(identity, generation, action).await else {
            return Ok(false);
        };
        if state.dirty {
            return self.discard_sparse_response(key).await;
        }
        if !self.generation_is_current(identity, generation).await {
            self.retire_stale_sparse_responses(key).await?;
            return Ok(false);
        }
        if state.committed || state.received == 0 {
            self.finish_sparse_intent(key, &state).await?;
            self.remove_sparse_state(action).await;
            return Ok(true);
        }
        self.commit_sparse_response(key, state).await
    }

    async fn discard_sparse_response(&self, key: &str) -> Result<bool> {
        let mut entries = self.entries.lock().await;
        self.discard(&mut entries, key).await?;
        Ok(false)
    }
}

fn validate_write(
    state: &SparseResponseState,
    action: &StoreAction,
    offset: u64,
    bytes: &[u8],
) -> Result<()> {
    ensure!(action.is_active(), "sparse action was revoked");
    ensure!(
        !state.committed && !state.dirty,
        "sparse action is not writable"
    );
    ensure!(
        offset == state.next_offset,
        "sparse response is not contiguous"
    );
    let end = offset
        .checked_add(bytes.len() as u64)
        .ok_or_else(|| anyhow::anyhow!("sparse response offset overflows"))?;
    ensure!(end <= state.range.end, "sparse response exceeds its range");
    Ok(())
}

mod commit;
mod recovery;
mod replacement;
mod state;
