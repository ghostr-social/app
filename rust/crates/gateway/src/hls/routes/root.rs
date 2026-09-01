use super::{fetch_manifest, manifest_response, prepared_manifest, CacheUse};
use crate::hls::sessions::{HlsPlaybackBinding, HlsSessionId};
use crate::router::GatewayHttpState;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{Response, StatusCode};
use reqwest::Url;
use std::sync::Arc;

pub(crate) async fn root_manifest(
    State(state): State<Arc<GatewayHttpState>>,
    Path(raw_session): Path<String>,
) -> Result<Response<Body>, StatusCode> {
    let session = HlsSessionId::parse(&raw_session).ok_or(StatusCode::NOT_FOUND)?;
    let binding = state
        .hls_sessions
        .playback_binding(&session)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
    let manifest = match binding {
        HlsPlaybackBinding::Prepared(asset) => prepared_root(&state, &session, &asset).await,
        HlsPlaybackBinding::Unprepared(sources) => unprepared_root(&state, &session, sources).await,
    }?;
    manifest_response(manifest)
}

async fn prepared_root(
    state: &GatewayHttpState,
    session: &HlsSessionId,
    asset: &ghostr_delivery::segmented::PreparedHlsPlaybackAsset,
) -> Result<String, StatusCode> {
    prepared_manifest(state, session, asset, asset.playback_manifest_source())
        .await
        .map_err(|error| {
            log::warn!("Prepared HLS manifest failed: {error:#}");
            StatusCode::BAD_GATEWAY
        })
}

async fn unprepared_root(
    state: &GatewayHttpState,
    session: &HlsSessionId,
    sources: Vec<Url>,
) -> Result<String, StatusCode> {
    for source in sources {
        if let Ok(manifest) = fetch_manifest(state, session, source, CacheUse::Fresh).await {
            return Ok(manifest);
        }
    }
    Err(StatusCode::BAD_GATEWAY)
}
