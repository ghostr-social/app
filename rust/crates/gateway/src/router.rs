#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
use crate::debug::http as debug_http;
use crate::progressive::route::{self as progressive_route, ProgressiveState};
use crate::{hls::routes as hls_routes, hls::sessions::HlsSessions};
use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;
#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
use ghostr_delivery::delivery_events::DeliveryHandle;
use ghostr_delivery::segmented::SegmentedCache;
use ghostr_net::media_request_executor::MediaRequestExecutor;
use ghostr_net::transfer_timeouts::HlsTransferTimeouts;
#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
use nostr_sdk::Client;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct GatewayHttpState {
    pub requests: MediaRequestExecutor,
    pub hls_sessions: HlsSessions,
    pub segmented: SegmentedCache,
    pub hls_timeouts: HlsTransferTimeouts,
}

#[derive(Clone)]
pub struct GatewayRouterResources {
    hls_sessions: HlsSessions,
    requests: MediaRequestExecutor,
    segmented: SegmentedCache,
}

impl GatewayRouterResources {
    pub fn new(hls_sessions: HlsSessions, requests: MediaRequestExecutor) -> Self {
        Self {
            hls_sessions,
            requests,
            segmented: SegmentedCache::new(),
        }
    }

    pub fn with_segmented(mut self, segmented: SegmentedCache) -> Self {
        self.segmented = segmented;
        self
    }
}

/// The gateway with `/video.mp4` served from the partial store instead of
/// proxied upstream; HLS routes and `/status` are wired unchanged.
pub fn configured_router_with_progressive(
    resources: GatewayRouterResources,
    progressive: Arc<ProgressiveState>,
) -> Router {
    configured_router_with_segmented(resources, progressive)
}

pub fn configured_router_with_segmented(
    resources: GatewayRouterResources,
    progressive: Arc<ProgressiveState>,
) -> Router {
    progressive_route::router(progressive).merge(shared_router(shared_state(resources)))
}

#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
pub fn configured_router_with_progressive_debug(
    resources: GatewayRouterResources,
    progressive: Arc<ProgressiveState>,
    delivery: DeliveryHandle,
    nostr: Arc<Client>,
) -> Router {
    configured_router_with_segmented_debug(resources, progressive, delivery, nostr)
}

#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
pub fn configured_router_with_segmented_debug(
    resources: GatewayRouterResources,
    progressive: Arc<ProgressiveState>,
    delivery: DeliveryHandle,
    nostr: Arc<Client>,
) -> Router {
    let debug = debug_http::DebugHttpResources {
        progressive: progressive.clone(),
        requests: resources.requests.clone(),
        delivery,
        hls: resources.hls_sessions.clone(),
        client: nostr,
    };
    progressive_route::router(progressive.clone())
        .merge(debug_http::router(debug))
        .merge(shared_router(shared_state(resources)))
}

fn shared_state(resources: GatewayRouterResources) -> Arc<GatewayHttpState> {
    Arc::new(GatewayHttpState {
        requests: resources.requests,
        hls_sessions: resources.hls_sessions,
        segmented: resources.segmented,
        hls_timeouts: HlsTransferTimeouts::default(),
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
