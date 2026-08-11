use super::ProgressiveState;
use crate::progressive::stream::StreamSource;
use axum::http::StatusCode;
use ghostr_engine::representation::RepresentationBinding;
use std::sync::Arc;
use tokio::time::{timeout_at, Instant};

pub(super) struct VideoSnapshot {
    pub(super) source: StreamSource,
    pub(super) total: u64,
}

pub(super) async fn awaited_snapshot(
    state: &Arc<ProgressiveState>,
    id: String,
) -> Result<Option<VideoSnapshot>, StatusCode> {
    let before = state.store.representation_binding(&id).await;
    let Some(total) = awaited_total_len(state, &id).await? else {
        return Ok(None);
    };
    let after = state.store.representation_binding(&id).await;
    let Some(binding) = stable_binding(before, after) else {
        return Ok(None);
    };
    Ok(Some(VideoSnapshot {
        source: StreamSource::new(id, binding),
        total,
    }))
}

fn stable_binding(
    before: Option<RepresentationBinding>,
    after: Option<RepresentationBinding>,
) -> Option<Option<RepresentationBinding>> {
    match (&before, &after) {
        (Some(left), Some(right)) if left != right => None,
        (Some(_), None) => None,
        _ => Some(after.or(before)),
    }
}

async fn awaited_total_len(
    state: &Arc<ProgressiveState>,
    id: &str,
) -> Result<Option<u64>, StatusCode> {
    let deadline = Instant::now() + state.timing.unknown_length_wait;
    let notify = state.store.change_notifier();
    loop {
        let changed = notify.notified();
        let known = state
            .store
            .total_len(id)
            .await
            .map_err(|_| StatusCode::NOT_FOUND)?;
        if known.is_some() {
            return Ok(known);
        }
        if timeout_at(deadline, changed).await.is_err() {
            return Ok(None);
        }
    }
}
