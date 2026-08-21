use axum::Router;
use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_delivery::delivery_events::{DeliveryFocus, FocusItem};
use ghostr_delivery::playback_demand::demand_channel;
use ghostr_delivery::progressive_posts::ServablePosts;
use ghostr_delivery::segmented::SegmentedCache;
use ghostr_engine::{DeliveryKind, PostId, VideoMeta};
use ghostr_gateway::hls::sessions::HlsSessions;
use ghostr_gateway::progressive::capabilities::ProgressiveCapabilities;
use ghostr_gateway::progressive::route::{ProgressiveState, ProgressiveTiming};
use ghostr_gateway::router::{configured_router_with_segmented, GatewayRouterResources};
use ghostr_net::media_request_executor::MediaRequestExecutor;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn router_with_hls(hls_sessions: HlsSessions, requests: MediaRequestExecutor) -> Router {
    router_with_segmented_hls(hls_sessions, requests, SegmentedCache::new())
}

pub fn router_with_segmented_hls(
    hls_sessions: HlsSessions,
    requests: MediaRequestExecutor,
    segmented: SegmentedCache,
) -> Router {
    let store = Arc::new(PartialRangeStore::with_capacity(
        super::temp_directory("hls-router"),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    ));
    let (demand, _) = demand_channel();
    let state = Arc::new(ProgressiveState {
        store,
        demand,
        cache: ServablePosts::new(),
        network: NetworkThrottle::new(),
        timing: ProgressiveTiming::default(),
        capabilities: ProgressiveCapabilities::production(),
        #[cfg(feature = "video-debug-web")]
        debug_feed: test_debug_feed(),
    });
    let resources = GatewayRouterResources::new(hls_sessions, requests).with_segmented(segmented);
    configured_router_with_segmented(resources, state)
}

pub fn hls_focus(source: &str) -> DeliveryFocus {
    DeliveryFocus::compatibility(
        vec![FocusItem {
            post: PostId::new("stream"),
            meta: VideoMeta {
                urls: vec![source.to_owned()],
                delivery: DeliveryKind::Hls,
                sha256: None,
                size_bytes: None,
                duration_ms: Some(4_000),
            },
        }],
        0,
        0,
    )
}

#[cfg(feature = "video-debug-web")]
fn test_debug_feed() -> ghostr_delivery::debug::feed::DebugFeed {
    let (delivery, _) = ghostr_delivery::delivery_events::command_channel();
    ghostr_delivery::debug::feed::DebugFeed::new(delivery, Vec::new())
}
