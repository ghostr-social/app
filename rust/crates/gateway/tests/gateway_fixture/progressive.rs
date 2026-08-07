use axum::Router;
use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_delivery::playback_demand::{demand_channel, DemandReceiver};
use ghostr_delivery::progressive_posts::ServablePosts;
#[cfg(feature = "video-debug-web")]
use ghostr_discovery::cache::client_with_event_cache;
use ghostr_gateway::hls::sessions::HlsSessions;
#[cfg(not(feature = "video-debug-web"))]
use ghostr_gateway::router::configured_router_with_progressive;
#[cfg(feature = "video-debug-web")]
use ghostr_gateway::router::configured_router_with_progressive_debug;
use ghostr_gateway::progressive::route::{ProgressiveState, ProgressiveTiming};
use ghostr_media_model::native_models::new_native_downloads;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::{path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

pub struct ProgressiveHarness {
    pub router: Router,
    pub store: Arc<PartialRangeStore>,
    pub posts: ServablePosts,
    pub network: NetworkThrottle,
    pub demand: DemandReceiver,
    pub root: PathBuf,
    #[cfg(feature = "video-debug-web")]
    pub debug_feed: ghostr_delivery::debug::feed::DebugFeed,
    #[cfg(feature = "video-debug-web")]
    pub hls_sessions: HlsSessions,
}

pub fn progressive_harness(prefix: &str) -> ProgressiveHarness {
    progressive_harness_with_timing(prefix, ProgressiveTiming::default())
}

pub fn video_request(id: &str, range: Option<&str>) -> axum::http::Request<axum::body::Body> {
    super::progressive_request::video_request(id, range)
}

pub fn progressive_harness_with_timing(
    prefix: &str,
    timing: ProgressiveTiming,
) -> ProgressiveHarness {
    let root = super::temp_directory(prefix);
    let store = Arc::new(PartialRangeStore::new(
        root.clone(),
        Arc::new(Mutex::new(0)),
    ));
    progressive_harness_with_store(root, store, timing)
}

pub fn progressive_harness_with_store(
    root: PathBuf,
    store: Arc<PartialRangeStore>,
    timing: ProgressiveTiming,
) -> ProgressiveHarness {
    let posts = ServablePosts::new();
    let (sender, demand) = demand_channel();
    let network = NetworkThrottle::new();
    #[cfg(feature = "video-debug-web")]
    let (debug_delivery, _) = ghostr_delivery::delivery_events::command_channel();
    #[cfg(feature = "video-debug-web")]
    let debug_feed =
        ghostr_delivery::debug::feed::DebugFeed::new(debug_delivery.clone(), Vec::new());
    #[cfg(feature = "video-debug-web")]
    let hls_sessions = HlsSessions::production();
    let state = Arc::new(ProgressiveState {
        store: store.clone(),
        demand: sender,
        cache: posts.clone(),
        network: network.clone(),
        timing,
        #[cfg(feature = "video-debug-web")]
        debug_feed: debug_feed.clone(),
    });
    #[cfg(not(feature = "video-debug-web"))]
    let router = configured_router_with_progressive(
        new_native_downloads(),
        HlsSessions::production(),
        super::media_client(),
        state,
    );
    #[cfg(feature = "video-debug-web")]
    let router = configured_router_with_progressive_debug(
        new_native_downloads(),
        hls_sessions.clone(),
        super::media_client(),
        state,
        debug_delivery,
        Arc::new(client_with_event_cache()),
    );
    ProgressiveHarness {
        router,
        store,
        posts,
        network,
        demand,
        root,
        #[cfg(feature = "video-debug-web")]
        debug_feed,
        #[cfg(feature = "video-debug-web")]
        hls_sessions,
    }
}
