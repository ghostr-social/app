use crate::hls::sessions::{HlsPlaybackBinding, HlsResourceId, HlsSessionId};
use crate::hls::transfer::HlsTransfer;
use crate::router::GatewayHttpState;
use anyhow::{bail, Result};
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header::{ACCEPT_ENCODING, CONTENT_LENGTH, CONTENT_TYPE};
use axum::http::{Response, StatusCode};
use ghostr_delivery::segmented::{CachedHlsGeneration, CachedHlsObject};
use ghostr_engine::adaptive::PreemptionAuthority;
use ghostr_hls_manifest::hls_manifest::HlsResourceKind;
use ghostr_hls_manifest::hls_manifest::MAX_HLS_MANIFEST_BYTES;
use reqwest::header::HeaderValue;
use reqwest::Url;
use std::sync::Arc;

pub(crate) use crate::hls::asset_delivery::asset;
mod root;
pub(crate) use root::root_manifest;

#[cfg(test)]
mod tests;

const HLS_CONTENT_TYPE: &str = "application/vnd.apple.mpegurl";

#[derive(Clone, Copy)]
enum CacheUse {
    Fresh,
    Existing,
}

pub(crate) async fn nested_manifest(
    State(state): State<Arc<GatewayHttpState>>,
    Path((raw_session, raw_resource)): Path<(String, String)>,
) -> Result<Response<Body>, StatusCode> {
    let (session, resource) = parsed_resource(&raw_session, &raw_resource)?;
    let resource = state
        .hls_sessions
        .resource(&session, resource)
        .await
        .filter(|item| item.kind == HlsResourceKind::Manifest)
        .ok_or(StatusCode::NOT_FOUND)?;
    let manifest = fetch_manifest(&state, &session, resource.url, CacheUse::Existing)
        .await
        .map_err(|error| {
            log::warn!("Nested HLS manifest fetch failed: {error:#}");
            StatusCode::BAD_GATEWAY
        })?;
    manifest_response(manifest)
}

async fn fetch_manifest(
    state: &GatewayHttpState,
    session: &HlsSessionId,
    source: Url,
    cache_use: CacheUse,
) -> Result<String> {
    if let Some(manifest) = prepared_manifest_if_bound(state, session, &source).await? {
        return Ok(manifest);
    }
    if let Some(manifest) = cached_manifest(state, session, &source, cache_use).await {
        return manifest;
    }
    let previous = state.segmented.object(source.as_str());
    let request = state
        .requests
        .get(source.as_str(), PreemptionAuthority::PlaybackCritical)?
        .header(ACCEPT_ENCODING, HeaderValue::from_static("identity"));
    let mut transfer = HlsTransfer::open(request, state.hls_timeouts).await?;
    if transfer.response().status() != StatusCode::OK {
        bail!("full HLS manifest response is not 200");
    }
    let final_url = transfer.response().url().clone();
    let body = transfer.read_bounded(MAX_HLS_MANIFEST_BYTES).await?;
    let observed =
        CachedHlsGeneration::for_response(&final_url, &body, transfer.response().headers());
    invalidate_changed(state, &source, previous.as_ref(), observed);
    state
        .hls_sessions
        .rewrite_manifest(session, &body, &final_url)
        .await
}

async fn prepared_manifest_if_bound(
    state: &GatewayHttpState,
    session: &HlsSessionId,
    source: &Url,
) -> Result<Option<String>> {
    let binding = state
        .hls_sessions
        .playback_binding(session)
        .await
        .ok_or_else(|| anyhow::anyhow!("secure HLS session is unavailable"))?;
    let HlsPlaybackBinding::Prepared(asset) = binding else {
        return Ok(None);
    };
    prepared_manifest(state, session, &asset, source.as_str())
        .await
        .map(Some)
}

async fn prepared_manifest(
    state: &GatewayHttpState,
    session: &HlsSessionId,
    asset: &ghostr_delivery::segmented::PreparedHlsPlaybackAsset,
    source: &str,
) -> Result<String> {
    let object = asset
        .object(source)
        .ok_or_else(|| anyhow::anyhow!("prepared HLS manifest is outside its pinned cohort"))?;
    state
        .hls_sessions
        .rewrite_manifest(session, &object.body, &object.final_url)
        .await
}

fn invalidate_changed(
    state: &GatewayHttpState,
    source: &Url,
    previous: Option<&CachedHlsObject>,
    observed: CachedHlsGeneration,
) {
    let Some(previous) = previous.filter(|object| object.generation() != observed) else {
        return;
    };
    state
        .segmented
        .invalidate_generation(source.as_str(), previous.generation());
}

async fn cached_manifest(
    state: &GatewayHttpState,
    session: &HlsSessionId,
    source: &Url,
    cache_use: CacheUse,
) -> Option<Result<String>> {
    let object = match cache_use {
        CacheUse::Fresh => state.segmented.reusable_object(source.as_str()),
        CacheUse::Existing => state.segmented.object(source.as_str()),
    }?;
    Some(
        state
            .hls_sessions
            .rewrite_manifest(session, &object.body, &object.final_url)
            .await,
    )
}

pub(super) fn parsed_resource(
    raw_session: &str,
    raw_resource: &str,
) -> Result<(HlsSessionId, HlsResourceId), StatusCode> {
    let session = HlsSessionId::parse(raw_session).ok_or(StatusCode::NOT_FOUND)?;
    let resource = HlsResourceId::parse(raw_resource).ok_or(StatusCode::NOT_FOUND)?;
    Ok((session, resource))
}

fn manifest_response(manifest: String) -> Result<Response<Body>, StatusCode> {
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, HLS_CONTENT_TYPE)
        .header(CONTENT_LENGTH, manifest.len())
        .body(Body::from(manifest))
        .map_err(|error| {
            log::warn!("Could not build HLS manifest response: {error}");
            StatusCode::INTERNAL_SERVER_ERROR
        })
}
