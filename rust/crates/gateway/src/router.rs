use ghostr_media_model::native_models::NativeDownloads;
use ghostr_net::outbound_media_client::MediaHttpClient;
use crate::progressive::route::{self as progressive_route, ProgressiveState};
#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
use crate::debug::http as debug_http;
#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
use ghostr_delivery::delivery_events::DeliveryHandle;
use crate::{hls::routes as hls_routes, hls::sessions::HlsSessions};
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
use nostr_sdk::Client;
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
    let state = shared_state(downloads, hls_sessions, client);
    Router::new()
        .route("/video.mp4", get(stream_video))
        .with_state(state.clone())
        .merge(shared_router(state))
}

/// The gateway with `/video.mp4` served from the partial store instead of
/// proxied upstream; HLS routes and `/status` are wired unchanged.
pub fn configured_router_with_progressive(
    downloads: NativeDownloads,
    hls_sessions: HlsSessions,
    client: MediaHttpClient,
    progressive: Arc<ProgressiveState>,
) -> Router {
    progressive_route::router(progressive).merge(shared_router(shared_state(
        downloads,
        hls_sessions,
        client,
    )))
}

#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
pub fn configured_router_with_progressive_debug(
    downloads: NativeDownloads,
    hls_sessions: HlsSessions,
    client: MediaHttpClient,
    progressive: Arc<ProgressiveState>,
    delivery: DeliveryHandle,
    nostr: Arc<Client>,
) -> Router {
    progressive_route::router(progressive.clone())
        .merge(debug_http::router(
            progressive,
            delivery,
            hls_sessions.clone(),
            nostr,
        ))
        .merge(shared_router(shared_state(downloads, hls_sessions, client)))
}

fn shared_state(
    downloads: NativeDownloads,
    hls_sessions: HlsSessions,
    client: MediaHttpClient,
) -> Arc<GatewayHttpState> {
    Arc::new(GatewayHttpState {
        client,
        downloads,
        hls_sessions,
    })
}

fn shared_router(state: Arc<GatewayHttpState>) -> Router {
    Router::new()
        .route("/status", get(gateway_status))
        .route(
            "/hls/{session}/index.m3u8",
            get(hls_routes::root_manifest),
        )
        .route(
            "/hls/{session}/manifests/{resource}/index.m3u8",
            get(hls_routes::nested_manifest),
        )
        .route(
            "/hls/{session}/assets/{resource}",
            get(hls_routes::asset),
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
