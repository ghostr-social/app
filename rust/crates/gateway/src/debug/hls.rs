//! Debug-web acquisition boundary for Rust-owned HLS playback sessions.

use ghostr_delivery::debug::feed::DebugFeed;
use crate::hls::sessions::{HlsSessionId, HlsSessions};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{delete, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct DebugHlsState {
    feed: DebugFeed,
    sessions: HlsSessions,
}

#[derive(Deserialize)]
struct AcquireRequest {
    id: String,
}

#[derive(Serialize)]
struct AcquireResponse {
    session_id: String,
    playback_url: String,
}

pub(crate) fn router(feed: DebugFeed, sessions: HlsSessions) -> Router {
    Router::new()
        .route("/debug/api/hls", post(acquire))
        .route("/debug/api/hls/{session}", delete(release))
        .with_state(DebugHlsState { feed, sessions })
}

async fn acquire(
    State(state): State<DebugHlsState>,
    Json(request): Json<AcquireRequest>,
) -> Result<(StatusCode, Json<AcquireResponse>), StatusCode> {
    let sources = state
        .feed
        .hls_sources(&request.id)
        .ok_or(StatusCode::NOT_FOUND)?;
    let id = state
        .sessions
        .acquire(sources)
        .await
        .map_err(|_| StatusCode::UNPROCESSABLE_ENTITY)?;
    let playback_url = format!("/hls/{}/index.m3u8", id.as_str());
    let session_id = id.as_str().to_owned();
    Ok((
        StatusCode::CREATED,
        Json(AcquireResponse {
            session_id,
            playback_url,
        }),
    ))
}

async fn release(
    State(state): State<DebugHlsState>,
    Path(raw_session): Path<String>,
) -> StatusCode {
    let Some(id) = HlsSessionId::parse(&raw_session) else {
        return StatusCode::NOT_FOUND;
    };
    match state.sessions.release(&id).await {
        true => StatusCode::NO_CONTENT,
        false => StatusCode::NOT_FOUND,
    }
}
