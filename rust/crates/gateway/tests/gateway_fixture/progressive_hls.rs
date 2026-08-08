use axum::Router;
use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_delivery::playback_demand::demand_channel;
use ghostr_delivery::progressive_posts::ServablePosts;
use ghostr_gateway::hls::sessions::HlsSessions;
use ghostr_gateway::progressive::route::{ProgressiveState, ProgressiveTiming};
use ghostr_gateway::router::configured_router_with_progressive;
use ghostr_net::outbound_media_client::MediaHttpRequests;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::sync::Arc;
use tokio::sync::Mutex;

pub fn router_with_hls(hls_sessions: HlsSessions, client: Arc<dyn MediaHttpRequests>) -> Router {
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
        #[cfg(feature = "video-debug-web")]
        debug_feed: test_debug_feed(),
    });
    configured_router_with_progressive(hls_sessions, client, state)
}

#[cfg(feature = "video-debug-web")]
fn test_debug_feed() -> ghostr_delivery::debug::feed::DebugFeed {
    let (delivery, _) = ghostr_delivery::delivery_events::command_channel();
    ghostr_delivery::debug::feed::DebugFeed::new(delivery, Vec::new())
}
