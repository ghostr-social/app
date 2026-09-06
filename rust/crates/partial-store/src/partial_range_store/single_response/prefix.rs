use super::{PartialRangeStore, SessionResponse, SingleResponseState};
use crate::partial_range_disk as disk;
use crate::partial_range_manifest::RangeManifest;
use anyhow::Result;
use core::ops::Range;
use ghostr_engine::adaptive::{WholeBodyContract, REQUEST_SLICE_BYTES};

mod exposure;
mod state;
use exposure::Exposure;
pub(super) use state::StagedPrefix;

impl PartialRangeStore {
    pub(in crate::partial_range_store) async fn has_readable_staged_prefix(
        &self,
        key: &str,
    ) -> bool {
        self.single_response_actions
            .lock()
            .await
            .get(key)
            .is_some_and(|state| state.owner.is_active() && state.readable_prefix().is_some())
    }

    pub(super) async fn publish_staged_prefix(
        &self,
        state: &SingleResponseState,
        span: Range<u64>,
    ) -> Result<()> {
        let key = state.identity.post().as_str();
        if !state.authority.exposes_staged_prefix() || span.is_empty() {
            return Ok(());
        }
        let prefix = self.extend_staged_prefix(state, span).await?;
        let exposure = self.staged_prefix_exposure(state, &prefix).await?;
        if !self
            .record_staged_prefix(state, exposure.prefix(prefix))
            .await
        {
            return Ok(());
        }
        if exposure == Exposure::NewRevision {
            self.advance_content_revision(key).await;
        }
        if exposure != Exposure::Pending {
            self.changed.notify_waiters();
        }
        Ok(())
    }

    async fn record_staged_prefix(
        &self,
        state: &SingleResponseState,
        prefix: StagedPrefix,
    ) -> bool {
        let mut actions = self.single_response_actions.lock().await;
        let Some(known) = actions
            .get_mut(state.identity.post().as_str())
            .filter(|known| known.owner.matches(state.owner.as_ref()))
        else {
            return false;
        };
        known.prefix = Some(prefix);
        true
    }

    async fn extend_staged_prefix(
        &self,
        state: &SingleResponseState,
        span: Range<u64>,
    ) -> Result<RangeManifest> {
        let mut prefix = state
            .prefix
            .as_ref()
            .map(StagedPrefix::manifest)
            .cloned()
            .unwrap_or_default();
        if let WholeBodyContract::Exact { expected_bytes } = state.contract {
            prefix.set_total_len(expected_bytes)?;
        }
        prefix.insert(span.clone())?;
        let covered = span.start / REQUEST_SLICE_BYTES * REQUEST_SLICE_BYTES..span.end;
        let path = self.paths.single_response(state.identity.post().as_str());
        for (range, checksum) in disk::checksum_blocks(&path, &[covered]).await? {
            prefix.record_checksum(range, checksum)?;
        }
        Ok(prefix)
    }

    pub(in crate::partial_range_store) async fn readable_session_response(
        &self,
        key: &str,
    ) -> Option<SessionResponse> {
        if let Some(response) = self.session_response(key).await {
            return Some(response);
        }
        let state = self
            .single_response_actions
            .lock()
            .await
            .get(key)
            .cloned()?;
        self.current_single_response(&state.identity, state.owner.as_ref())
            .await?;
        SessionResponse::active(&state)
    }

    pub(in crate::partial_range_store) async fn discard_readable_session(
        &self,
        key: &str,
        response: &SessionResponse,
    ) -> Result<()> {
        if response.is_complete() {
            return self.discard_session_response(key).await;
        }
        self.revoke_single_response(key).await;
        self.cancel_single_response(key).await
    }
}
