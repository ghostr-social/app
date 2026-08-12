use axum::Router;
use ghostr_delivery::debug::network::NetworkThrottle;
#[cfg(feature = "video-debug-web")]
use ghostr_delivery::delivery_events::CommandReceiver;
use ghostr_delivery::playback_demand::DemandReceiver;
use ghostr_delivery::progressive_posts::ServablePosts;
#[cfg(feature = "video-debug-web")]
use ghostr_gateway::hls::sessions::HlsSessions;
use ghostr_gateway::progressive::capabilities::ProgressiveCapabilities;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::{path::PathBuf, sync::Arc};

pub struct ProgressiveHarness {
    pub router: Router,
    pub store: Arc<PartialRangeStore>,
    pub posts: ServablePosts,
    pub network: NetworkThrottle,
    pub capabilities: ProgressiveCapabilities,
    pub demand: DemandReceiver,
    pub root: PathBuf,
    #[cfg(feature = "video-debug-web")]
    pub debug_feed: ghostr_delivery::debug::feed::DebugFeed,
    #[cfg(feature = "video-debug-web")]
    pub hls_sessions: HlsSessions,
    #[cfg(feature = "video-debug-web")]
    pub debug_commands: CommandReceiver,
}
