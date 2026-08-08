use crate::hls::sessions::{HlsResourceId, HlsSessionId};
use crate::router::{proxy_response, upstream_request, GatewayHttpState};
use anyhow::{bail, Context, Result};
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header::{CONTENT_LENGTH, CONTENT_TYPE};
use axum::http::{HeaderMap, Response, StatusCode};
use ghostr_hls_manifest::hls_manifest::HlsResourceKind;
use ghostr_hls_manifest::hls_manifest::MAX_HLS_MANIFEST_BYTES;
use reqwest::Url;
use std::sync::Arc;
use std::time::Duration;

const HLS_MANIFEST_TIMEOUT: Duration = Duration::from_secs(30);
const HLS_CONTENT_TYPE: &str = "application/vnd.apple.mpegurl";

pub(crate) async fn root_manifest(
    State(state): State<Arc<GatewayHttpState>>,
    Path(raw_session): Path<String>,
) -> Result<Response<Body>, StatusCode> {
    let session = HlsSessionId::parse(&raw_session).ok_or(StatusCode::NOT_FOUND)?;
    let sources = state
        .hls_sessions
        .sources(&session)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
    for source in sources {
        if let Ok(manifest) = fetch_manifest(&state, &session, source).await {
            return manifest_response(manifest);
        }
    }
    Err(StatusCode::BAD_GATEWAY)
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
    let manifest = fetch_manifest(&state, &session, resource.url)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    manifest_response(manifest)
}

pub(crate) async fn asset(
    State(state): State<Arc<GatewayHttpState>>,
    Path((raw_session, raw_resource)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Response<Body>, StatusCode> {
    let (session, resource) = parsed_resource(&raw_session, &raw_resource)?;
    let resource = state
        .hls_sessions
        .resource(&session, resource)
        .await
        .filter(|item| item.kind == HlsResourceKind::Asset)
        .ok_or(StatusCode::NOT_FOUND)?;
    let upstream = upstream_request(&state.client, resource.url.to_string(), &headers)?
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    proxy_response(upstream)
}

async fn fetch_manifest(
    state: &GatewayHttpState,
    session: &HlsSessionId,
    source: Url,
) -> Result<String> {
    tokio::time::timeout(
        HLS_MANIFEST_TIMEOUT,
        fetch_manifest_inner(state, session, source),
    )
    .await
    .context("HLS manifest timed out")?
}

async fn fetch_manifest_inner(
    state: &GatewayHttpState,
    session: &HlsSessionId,
    source: Url,
) -> Result<String> {
    let mut response = state
        .client
        .get(source.as_str())?
        .send()
        .await?
        .error_for_status()?;
    require_hls_mime(response.headers())?;
    let final_url = response.url().clone();
    let body = bounded_manifest(&mut response).await?;
    state
        .hls_sessions
        .rewrite_manifest(session, &body, &final_url)
        .await
}

async fn bounded_manifest(response: &mut reqwest::Response) -> Result<Vec<u8>> {
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if body.len().saturating_add(chunk.len()) > MAX_HLS_MANIFEST_BYTES {
            bail!("HLS manifest exceeds its byte limit");
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

fn require_hls_mime(headers: &HeaderMap) -> Result<()> {
    let value = headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .map(str::trim)
        .map(str::to_ascii_lowercase);
    if !matches!(
        value.as_deref(),
        Some(
            "application/vnd.apple.mpegurl"
                | "application/x-mpegurl"
                | "audio/mpegurl"
                | "audio/x-mpegurl"
        )
    ) {
        bail!("upstream response is not an HLS manifest");
    }
    Ok(())
}

fn parsed_resource(
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
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
