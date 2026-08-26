use super::snapshot::{SnapshotCapture, SnapshotProjection};
use super::{provisional, session, PartialRangeStore, StoredMediaSnapshot};
use crate::partial_range_paths::validate_key;
use anyhow::Result;
use core::ops::Range;

impl PartialRangeStore {
    /// One generation-coherent observation for planning and projection.
    ///
    /// # Errors
    ///
    /// Returns an error when the key is invalid or a coherent snapshot cannot be read.
    pub async fn media_snapshot(&self, key: &str) -> Result<StoredMediaSnapshot> {
        validate_key(key)?;
        let _update = self.observe_key(key).await?;
        if let Some(response) = self.session_response(key).await {
            return session::snapshot(self, key, &response).await;
        }
        let provisional = provisional::capture(self, key).await;
        let capture = self.snapshot_capture(key, &provisional).await?;
        let projection = self.snapshot_projection(key, &capture).await;
        Ok(capture.into_snapshot(projection))
    }

    async fn snapshot_capture(
        &self,
        key: &str,
        provisional: &[provisional::ProvisionalInterval],
    ) -> Result<SnapshotCapture> {
        let mut entries = self.entries.lock().await;
        let entry = self.entry(&mut entries, key).await?;
        let readable = provisional::merge(&entry.manifest, provisional)?;
        let finalized = entry.completion.is_some();
        Ok(SnapshotCapture::new(&entry.manifest, &readable, finalized))
    }

    async fn snapshot_projection(
        &self,
        key: &str,
        capture: &SnapshotCapture,
    ) -> SnapshotProjection {
        let response_storage = self
            .single_response_actions
            .lock()
            .await
            .get(key)
            .map(|state| state.storage);
        let planning_ranges = capture.planning_ranges(response_storage);
        let continuation_source = self
            .snapshot_continuation_source(key, &planning_ranges, capture.complete())
            .await;
        SnapshotProjection {
            binding: self.representations.lock().await.get(key).cloned(),
            revision: self.current_content_revision(key).await,
            planning_ranges,
            continuation_source,
        }
    }

    async fn snapshot_continuation_source(
        &self,
        key: &str,
        planning_ranges: &[Range<u64>],
        complete: bool,
    ) -> Option<String> {
        if planning_ranges.is_empty() || complete {
            return None;
        }
        self.source_generations
            .lock()
            .await
            .get(key)
            .map(|(source, _)| source.clone())
    }
}
