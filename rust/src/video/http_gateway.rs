use crate::video::native_models::NativeDownloads;
use crate::video::outbound_media_client::MediaHttpClient;
use crate::video::{hls_http_gateway, hls_sessions::HlsSessions};
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use reqwest::RequestBuilder;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct GatewayHttpState {
    pub client: MediaHttpClient,
    pub downloads: NativeDownloads,
    pub hls_sessions: HlsSessions,
}

#[derive(Deserialize)]
struct VideoQuery {
    id: String,
}

pub fn configured_router(downloads: NativeDownloads) -> anyhow::Result<Router> {
    configured_router_with_hls_sessions(downloads, HlsSessions::production())
}

pub fn configured_router_with_hls_sessions(
    downloads: NativeDownloads,
    hls_sessions: HlsSessions,
) -> anyhow::Result<Router> {
    Ok(configured_router_with_hls_client(
        downloads,
        hls_sessions,
        MediaHttpClient::public()?,
    ))
}

pub fn configured_router_with_client(
    downloads: NativeDownloads,
    client: MediaHttpClient,
) -> Router {
    configured_router_with_hls_client(downloads, HlsSessions::production(), client)
}

pub fn configured_router_with_hls_client(
    downloads: NativeDownloads,
    hls_sessions: HlsSessions,
    client: MediaHttpClient,
) -> Router {
    let state = Arc::new(GatewayHttpState {
        client,
        downloads,
        hls_sessions,
    });
    Router::new()
        .route("/status", get(gateway_status))
        .route("/video.mp4", get(stream_video))
        .route(
            "/hls/{session}/index.m3u8",
            get(hls_http_gateway::root_manifest),
        )
        .route(
            "/hls/{session}/manifests/{resource}/index.m3u8",
            get(hls_http_gateway::nested_manifest),
        )
        .route(
            "/hls/{session}/assets/{resource}",
            get(hls_http_gateway::asset),
        )
        .with_state(state)
}

async fn gateway_status() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn stream_video(
    State(state): State<Arc<GatewayHttpState>>,
    Query(query): Query<VideoQuery>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    let url = video_url(&state.downloads, &query.id)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
    let upstream = upstream_request(&state.client, url, &headers)?
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;
    proxy_response(upstream)
}

async fn video_url(downloads: &NativeDownloads, id: &str) -> Option<String> {
    downloads
        .lock()
        .await
        .get(id)
        .map(|video| video.url.clone())
}

pub(crate) fn upstream_request(
    client: &MediaHttpClient,
    url: String,
    headers: &HeaderMap,
) -> Result<RequestBuilder, StatusCode> {
    let request = client.get(&url).map_err(|_| StatusCode::BAD_GATEWAY)?;
    Ok(match headers.get(RANGE) {
        Some(value) => request.header(RANGE, value),
        None => request,
    })
}

pub(crate) fn proxy_response(upstream: reqwest::Response) -> Result<Response, StatusCode> {
    let status = upstream.status();
    let headers = upstream.headers().clone();
    let mut response = Response::builder().status(status);
    for name in [CONTENT_TYPE, CONTENT_LENGTH, CONTENT_RANGE, ACCEPT_RANGES] {
        if let Some(value) = headers.get(&name) {
            response = response.header(name, value);
        }
    }
    response
        .body(Body::from_stream(upstream.bytes_stream()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
