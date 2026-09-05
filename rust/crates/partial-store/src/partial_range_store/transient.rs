//! Bounded, volatile whole-response playback buffers for no-store media.
//! Neither verified content hashes nor a normal EOF promote them to disk.
use super::single_response::{
    ResponseOwner, SingleResponseAuthority, SingleResponseState, SingleResponseStorage,
};
use super::{Entries, PartialRangeStore, ResponseOpenResult, StoreAction};
use anyhow::{ensure, Context as _, Result};
use ghostr_engine::adaptive::WholeBodyContract;
use ghostr_engine::representation::TransferIdentity;
use sha2::{Digest as _, Sha256};

mod reads;
mod writes;

const MAX_OBJECT_BYTES: u64 = 16 * 1024 * 1024;
const MAX_POOL_BYTES: u64 = 32 * 1024 * 1024;
const MAX_OBJECTS: usize = 32;

pub(super) struct TransientResponse {
    bytes: Vec<u8>,
    limit: u64,
    complete: bool,
    digest: Sha256,
}

impl PartialRangeStore {
    /// # Errors
    /// Rejects stale actions, excessive RAM reservations, or failed removal of old disk state.
    pub async fn open_transient_single_response(
        &self,
        identity: &TransferIdentity,
        action: &StoreAction,
        contract: WholeBodyContract,
    ) -> Result<ResponseOpenResult> {
        let key = identity.post().as_str();
        let _update = self.update_key(key).await?;
        if !action.is_active() || action.identity() != identity {
            return Ok(ResponseOpenResult::Stale);
        }
        self.current_binding(identity).await?;
        if self.single_response_is_active(key).await {
            return Ok(ResponseOpenResult::RequiresIndependentObject);
        }
        let mut entries = self.entries.lock().await;
        self.discard(&mut entries, key).await?;
        self.reserve_transient(&mut entries, key, contract).await?;
        self.register_transient(identity, action, contract).await;
        Ok(ResponseOpenResult::Opened)
    }

    async fn reserve_transient(
        &self,
        entries: &mut Entries,
        key: &str,
        contract: WholeBodyContract,
    ) -> Result<()> {
        let limit = contract.maximum_bytes();
        ensure!(
            limit > 0 && limit <= MAX_OBJECT_BYTES,
            "transient object exceeds RAM envelope"
        );
        self.make_transient_room(entries, limit).await?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(limit as usize)
            .context("reserve transient playback RAM")?;
        let entry = self.entry(entries, key).await?;
        if let WholeBodyContract::Exact { expected_bytes } = contract {
            entry.manifest.set_total_len(expected_bytes)?;
        }
        self.transient_responses.lock().await.insert(
            key.to_owned(),
            TransientResponse {
                bytes,
                limit,
                complete: false,
                digest: Sha256::new(),
            },
        );
        Ok(())
    }

    async fn register_transient(
        &self,
        identity: &TransferIdentity,
        action: &StoreAction,
        contract: WholeBodyContract,
    ) {
        self.single_response_actions.lock().await.insert(
            identity.post().as_str().to_owned(),
            SingleResponseState {
                identity: identity.clone(),
                owner: ResponseOwner::Granted(action.clone()),
                contract,
                authority: SingleResponseAuthority::ActionScoped,
                storage: SingleResponseStorage::Memory,
            },
        );
        self.selected()
            .insert(identity.post().as_str().to_owned(), identity.clone());
        self.changed.notify_waiters();
    }

    async fn make_transient_room(&self, entries: &mut Entries, wanted: u64) -> Result<()> {
        let reserved = self.reserved_keys().await;
        let mut candidates: Vec<_> = self
            .transient_responses
            .lock()
            .await
            .keys()
            .filter(|key| !self.leases.held(key) && !reserved.contains(*key))
            .map(|key| {
                (
                    entries.get(key).map_or(0, |entry| entry.touched),
                    key.clone(),
                )
            })
            .collect();
        candidates.sort();
        for (_, key) in candidates {
            if self.transient_room(wanted).await {
                return Ok(());
            }
            self.discard(entries, &key).await?;
        }
        ensure!(
            self.transient_room(wanted).await,
            "transient playback RAM is fully leased"
        );
        Ok(())
    }

    async fn transient_room(&self, wanted: u64) -> bool {
        let responses = self.transient_responses.lock().await;
        responses.len() < MAX_OBJECTS
            && responses
                .values()
                .map(|response| response.limit)
                .sum::<u64>()
                + wanted
                <= MAX_POOL_BYTES
    }
}
