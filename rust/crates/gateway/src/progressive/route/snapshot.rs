use super::ProgressiveState;
use crate::progressive::stream::StreamSource;
use axum::http::StatusCode;
use ghostr_partial_store::partial_range_store::StoredMediaSnapshot;
use std::sync::Arc;
use tokio::time::{timeout_at, Instant};

pub(super) struct VideoSnapshot {
    pub(super) source: StreamSource,
    pub(super) total: u64,
}

impl VideoSnapshot {
    pub(super) fn from_stored(id: String, snapshot: &StoredMediaSnapshot) -> Self {
        let total = snapshot.total_len().expect("awaited snapshot has a total");
        Self {
            source: StreamSource::new(id, snapshot.binding().cloned(), snapshot.revision()),
            total,
        }
    }
}

pub(super) async fn awaited_media_snapshot(
    state: &Arc<ProgressiveState>,
    id: &str,
) -> Result<Option<StoredMediaSnapshot>, StatusCode> {
    let deadline = Instant::now() + state.timing.unknown_length_wait;
    let notify = state.store.change_notifier();
    loop {
        let changed = notify.notified();
        tokio::pin!(changed);
        changed.as_mut().enable();
        let snapshot = state.store.media_snapshot(id).await.map_err(|error| {
            log::warn!("Could not inspect progressive media snapshot: {error:#}");
            StatusCode::NOT_FOUND
        })?;
        if snapshot.binding().is_none() {
            return Ok(None);
        }
        if snapshot.total_len().is_some() {
            return Ok(Some(snapshot));
        }
        if timeout_at(deadline, changed).await.is_err() {
            return Ok(None);
        }
    }
}
