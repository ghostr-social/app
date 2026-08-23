use super::{PartialRangeStore, SingleResponseState};
use crate::partial_range_disk as disk;
use crate::partial_range_manifest::RangeManifest;
use anyhow::{ensure, Result};
use ghostr_engine::representation::{RepresentationBinding, TransferIdentity};
use std::collections::HashMap;

mod promotion;

#[derive(Clone, Eq, PartialEq)]
pub(in crate::partial_range_store) struct SessionResponse {
    identity: TransferIdentity,
    manifest: RangeManifest,
}

impl SessionResponse {
    pub(in crate::partial_range_store) fn identity(&self) -> &TransferIdentity {
        &self.identity
    }

    pub(in crate::partial_range_store) fn manifest(&self) -> &RangeManifest {
        &self.manifest
    }

    pub(in crate::partial_range_store) fn bytes(&self) -> u64 {
        self.manifest.covered_bytes()
    }
}

impl PartialRangeStore {
    pub(in crate::partial_range_store) async fn promote_verified_session(
        &self,
        entries: &mut crate::partial_range_store::Entries,
        key: &str,
        response: &SessionResponse,
        digest: String,
    ) -> Result<crate::partial_range_completion::Completion> {
        promotion::publish(self, entries, key, response, digest).await
    }

    pub(super) async fn commit_session_response(
        &self,
        binding: &RepresentationBinding,
        state: &SingleResponseState,
        total: u64,
    ) -> Result<()> {
        let key = binding.post().as_str();
        ensure!(
            disk::file_len(&self.paths.single_response(key)).await? == Some(total),
            "session response length does not match its framing"
        );
        let manifest = self.session_manifest(key, total).await?;
        disk::save_manifest(&self.paths.single_response_manifest(key), &manifest).await?;
        let response = SessionResponse {
            identity: state.identity.clone(),
            manifest,
        };
        self.session_responses
            .lock()
            .await
            .insert(key.to_owned(), response);
        self.advance_content_revision(key).await;
        self.changed.notify_waiters();
        Ok(())
    }

    async fn session_manifest(&self, key: &str, total: u64) -> Result<RangeManifest> {
        super::staged::manifest::complete(&self.paths.single_response(key), total).await
    }

    pub(in crate::partial_range_store) async fn session_response(
        &self,
        key: &str,
    ) -> Option<SessionResponse> {
        self.session_responses.lock().await.get(key).cloned()
    }

    pub(super) async fn session_response_bytes(&self) -> HashMap<String, u64> {
        self.session_responses
            .lock()
            .await
            .iter()
            .map(|(key, response)| (key.clone(), response.bytes()))
            .collect()
    }

    pub(in crate::partial_range_store) async fn take_session_response(&self, key: &str) -> u64 {
        self.session_responses
            .lock()
            .await
            .remove(key)
            .map_or(0, |response| response.bytes())
    }

    pub(in crate::partial_range_store) async fn discard_session_response(
        &self,
        key: &str,
    ) -> Result<()> {
        let Some(response) = self.session_response(key).await else {
            return Ok(());
        };
        disk::remove_if_present(&self.paths.single_response(key)).await?;
        disk::remove_if_present(&self.paths.single_response_manifest(key)).await?;
        self.session_responses.lock().await.remove(key);
        self.release(response.bytes()).await;
        self.advance_content_revision(key).await;
        self.changed.notify_waiters();
        Ok(())
    }
}
