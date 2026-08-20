use super::{ProgressiveState, VideoQuery};
use axum::http::StatusCode;
use ghostr_partial_store::partial_range_store::StoredMediaSnapshot;

pub(super) async fn require_servable(
    state: &ProgressiveState,
    query: &VideoQuery,
) -> Result<(), StatusCode> {
    if recognized(state, query).await && state.cache.contains(&query.id) {
        return Ok(());
    }
    Err(StatusCode::NOT_FOUND)
}

pub(super) async fn refresh_current_asset(
    state: &ProgressiveState,
    query: &VideoQuery,
) -> Result<bool, StatusCode> {
    let snapshot = state
        .store
        .media_snapshot(&query.id)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    if snapshot.binding().is_none() {
        return Ok(false);
    }
    require_current_asset(state, query, &snapshot).await?;
    Ok(true)
}

pub(super) async fn require_current_asset(
    state: &ProgressiveState,
    query: &VideoQuery,
    snapshot: &StoredMediaSnapshot,
) -> Result<(), StatusCode> {
    let Some(binding) = snapshot.binding() else {
        return Err(StatusCode::NOT_FOUND);
    };
    if !state.cache.allows_binding(&query.id, binding) {
        return Err(StatusCode::NOT_FOUND);
    }
    let current = match query.cap.as_deref() {
        Some(capability) => {
            state
                .capabilities
                .authorizes(capability, &query.id, snapshot)
                .await
        }
        None => false,
    };
    current.then_some(()).ok_or(StatusCode::NOT_FOUND)
}

async fn recognized(state: &ProgressiveState, query: &VideoQuery) -> bool {
    match query.cap.as_deref() {
        Some(capability) => state.capabilities.recognizes(capability, &query.id).await,
        None => false,
    }
}
