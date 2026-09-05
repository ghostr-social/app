use super::{PartialRangeStore, ResponseOwnerRef, SingleResponseState, SingleResponseStorage};
use anyhow::Result;
use ghostr_engine::representation::TransferIdentity;
use std::collections::BTreeMap;

impl PartialRangeStore {
    pub(super) async fn current_single_response(
        &self,
        identity: &TransferIdentity,
        owner: ResponseOwnerRef<'_>,
    ) -> Option<SingleResponseState> {
        let state = self
            .single_response_actions
            .lock()
            .await
            .get(identity.post().as_str())
            .cloned()
            .filter(|state| {
                state.owner.is_active() && state.owner.matches(owner) && state.identity == *identity
            })?;
        if let Some(generation) = state.authority.generation() {
            if !self.http_generation_is_current(identity, generation).await {
                return None;
            }
        }
        self.current_binding(identity).await.ok().map(|_| state)
    }

    pub(super) async fn single_response_for_finish(
        &self,
        identity: &TransferIdentity,
        owner: ResponseOwnerRef<'_>,
    ) -> Option<SingleResponseState> {
        let state = self
            .single_response_actions
            .lock()
            .await
            .get(identity.post().as_str())
            .cloned()
            .filter(|state| state.owner.matches(owner) && state.identity == *identity)?;
        if let Some(generation) = state.authority.generation() {
            if !self.http_generation_is_current(identity, generation).await {
                return None;
            }
        }
        self.current_binding(identity).await.ok().map(|_| state)
    }

    pub(in crate::partial_range_store) async fn cancel_single_response(
        &self,
        key: &str,
    ) -> Result<()> {
        let state = self.single_response_actions.lock().await.get(key).cloned();
        if let Some(state) = state.as_ref() {
            if matches!(state.storage, SingleResponseStorage::Memory) {
                self.abort_transient(key).await?;
            }
            if matches!(state.storage, SingleResponseStorage::Staged { .. }) {
                self.remove_staged_single_response(state).await?;
            }
        }
        if let Some(state) = state {
            let mut actions = self.single_response_actions.lock().await;
            if actions
                .get(key)
                .is_some_and(|known| known.owner.matches(state.owner.as_ref()))
            {
                actions.remove(key);
            }
        }
        Ok(())
    }

    pub(in crate::partial_range_store) async fn revoke_single_response(&self, key: &str) {
        if let Some(state) = self.single_response_actions.lock().await.get(key) {
            state.owner.revoke();
        }
    }

    pub(in crate::partial_range_store) async fn live_single_response_started(
        &self,
        key: &str,
    ) -> bool {
        self.single_response_actions
            .lock()
            .await
            .get(key)
            .is_some_and(|state| {
                state.owner.is_active()
                    && matches!(state.storage, SingleResponseStorage::Live { started: true })
            })
    }

    pub(in crate::partial_range_store) async fn single_response_is_active(
        &self,
        key: &str,
    ) -> bool {
        self.single_response_actions
            .lock()
            .await
            .get(key)
            .is_some_and(|state| state.owner.is_active())
    }

    pub(in crate::partial_range_store) async fn live_single_response_is_active(
        &self,
        key: &str,
    ) -> bool {
        self.single_response_actions
            .lock()
            .await
            .get(key)
            .is_some_and(|state| {
                state.owner.is_active()
                    && matches!(state.storage, SingleResponseStorage::Live { .. })
            })
    }

    pub(in crate::partial_range_store) async fn staged_response_bytes(
        &self,
    ) -> BTreeMap<String, u64> {
        let mut staged: BTreeMap<_, _> = self
            .single_response_actions
            .lock()
            .await
            .iter()
            .filter_map(|(key, state)| match state.storage {
                SingleResponseStorage::Staged { received } => Some((key.clone(), received)),
                SingleResponseStorage::Live { .. } | SingleResponseStorage::Memory => None,
            })
            .collect();
        for (key, bytes) in self.session_response_bytes().await {
            *staged.entry(key).or_default() += bytes;
        }
        staged
    }

    pub(in crate::partial_range_store) async fn retry_inactive_single_response(&self, key: &str) {
        let _update = self.update_key_raw(key).await;
        if let Err(error) = self.retry_inactive_single_response_locked(key).await {
            log::warn!("Could not retry abandoned video response: {error:#}");
        }
    }

    pub(super) async fn retry_inactive_single_response_locked(&self, key: &str) -> Result<()> {
        let inactive = self
            .single_response_actions
            .lock()
            .await
            .get(key)
            .is_some_and(|state| !state.owner.is_active());
        if inactive {
            self.cancel_single_response(key).await?;
        }
        Ok(())
    }
}
