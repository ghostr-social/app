#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
use crate::debug::http as debug_http;
use crate::progressive::route::{self as progressive_route, ProgressiveState};
use crate::{hls::routes as hls_routes, hls::sessions::HlsSessions};
use axum::body::Body;
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
use ghostr_delivery::delivery_events::DeliveryHandle;
use ghostr_delivery::segmented::SegmentedCache;
use ghostr_net::outbound_media_client::MediaHttpRequests;
#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
use nostr_sdk::Client;
use reqwest::RequestBuilder;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct GatewayHttpState {
    pub client: Arc<dyn MediaHttpRequests>,
    pub hls_sessions: HlsSessions,
    pub segmented: SegmentedCache,
}

/// The gateway with `/video.mp4` served from the partial store instead of
/// proxied upstream; HLS routes and `/status` are wired unchanged.
pub fn configured_router_with_progressive(
    hls_sessions: HlsSessions,
    client: Arc<dyn MediaHttpRequests>,
    progressive: Arc<ProgressiveState>,
) -> Router {
    configured_router_with_segmented(hls_sessions, client, progressive, SegmentedCache::new())
}

pub fn configured_router_with_segmented(
    hls_sessions: HlsSessions,
    client: Arc<dyn MediaHttpRequests>,
    progressive: Arc<ProgressiveState>,
    segmented: SegmentedCache,
) -> Router {
    progressive_route::router(progressive).merge(shared_router(shared_state(
        hls_sessions,
        client,
        segmented,
    )))
}

#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
pub fn configured_router_with_progressive_debug(
    hls_sessions: HlsSessions,
    client: Arc<dyn MediaHttpRequests>,
    progressive: Arc<ProgressiveState>,
    delivery: DeliveryHandle,
    nostr: Arc<Client>,
) -> Router {
    configured_router_with_segmented_debug(
        hls_sessions,
        client,
        progressive,
        delivery,
        nostr,
        SegmentedCache::new(),
    )
}

#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
pub fn configured_router_with_segmented_debug(
    hls_sessions: HlsSessions,
    client: Arc<dyn MediaHttpRequests>,
    progressive: Arc<ProgressiveState>,
    delivery: DeliveryHandle,
    nostr: Arc<Client>,
    segmented: SegmentedCache,
) -> Router {
    progressive_route::router(progressive.clone())
        .merge(debug_http::router(
            progressive,
            delivery,
            hls_sessions.clone(),
            nostr,
        ))
        .merge(shared_router(shared_state(hls_sessions, client, segmented)))
}

fn shared_state(
    hls_sessions: HlsSessions,
    client: Arc<dyn MediaHttpRequests>,
    segmented: SegmentedCache,
) -> Arc<GatewayHttpState> {
    Arc::new(GatewayHttpState {
        client,
        hls_sessions,
        segmented,
    })
}

fn shared_router(state: Arc<GatewayHttpState>) -> Router {
    Router::new()
        .route("/status", get(gateway_status))
        .route("/hls/{session}/index.m3u8", get(hls_routes::root_manifest))
        .route(
            "/hls/{session}/manifests/{resource}/index.m3u8",
            get(hls_routes::nested_manifest),
        )
        .route("/hls/{session}/assets/{resource}", get(hls_routes::asset))
        .with_state(state)
}

async fn gateway_status() -> StatusCode {
    StatusCode::NO_CONTENT
}

pub(crate) fn upstream_request(
    client: &dyn MediaHttpRequests,
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
