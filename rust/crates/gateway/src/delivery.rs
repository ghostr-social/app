//! Wiring the progressive delivery half of the gateway: the range store
//! the router serves from, the downloader that fills it, and the
//! manager that decides what to fetch.

use crate::hls::sessions::HlsSessions;
use crate::progressive::capabilities::ProgressiveCapabilities;
use crate::progressive::route::{ProgressiveState, ProgressiveTiming};
#[cfg(not(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
)))]
use crate::router::configured_router_with_segmented;
#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
use crate::router::configured_router_with_segmented_debug;
use crate::runtime::GatewayConfiguration;
use ghostr_delivery::cache_registry::CacheRegistry;
#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
use ghostr_delivery::debug::feed::DebugFeed;
use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_delivery::delivery_events::DeliveryHandle;
use ghostr_delivery::manager::{
    start_delivery_manager_with_discovery_demand, DeliveryManagerConfig, DeliveryTuning,
};
use ghostr_delivery::playback_demand::demand_channel;
use ghostr_delivery::segmented::SegmentedCache;
use ghostr_engine::adaptive::DiscoveryDemand;
use ghostr_engine::{DataUsageLevel, EngineParams};
use ghostr_net::outbound_media_client::MediaHttpRequests;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use log::warn;
use nostr_sdk::Client;
use std::sync::Arc;
use tokio::sync::{watch, Mutex};

pub(crate) type DeliveryParts = (
    axum::Router,
    DeliveryHandle,
    Arc<ProgressiveState>,
    SegmentedCache,
    watch::Receiver<DiscoveryDemand>,
);

struct DeliveryResources {
    store: Arc<PartialRangeStore>,
    client: Arc<dyn MediaHttpRequests>,
    cache: CacheRegistry,
    network: NetworkThrottle,
    segmented: SegmentedCache,
}

/// Progressive delivery: the router serves `/video.mp4` from the partial
/// store; adaptive candidate demand feeds the discovery control loop.
pub(crate) async fn start_progressive_delivery(
    configuration: &GatewayConfiguration,
    hls_sessions: HlsSessions,
    nostr: Arc<Client>,
    client: Arc<dyn MediaHttpRequests>,
) -> anyhow::Result<DeliveryParts> {
    let store = Arc::new(opened_store(configuration).await);
    let cache = CacheRegistry::new();
    let (demand_sender, demand) = demand_channel();
    let network = NetworkThrottle::new();
    let segmented = SegmentedCache::new();
    let resources = DeliveryResources {
        store: store.clone(),
        client: client.clone(),
        cache: cache.clone(),
        network: network.clone(),
        segmented: segmented.clone(),
    };
    let config = delivery_config(configuration, resources);
    let (delivery, discovery_demand) = start_delivery_manager_with_discovery_demand(config, demand);
    let progressive = Arc::new(ProgressiveState {
        store: store.clone(),
        demand: demand_sender,
        cache,
        network,
        timing: ProgressiveTiming::default(),
        capabilities: ProgressiveCapabilities::production(),
        #[cfg(all(
            feature = "video-debug-web",
            debug_assertions,
            not(any(target_os = "android", target_os = "ios"))
        ))]
        debug_feed: DebugFeed::new(delivery.clone(), configuration.relays.clone()),
    });
    #[cfg(all(
        feature = "video-debug-web",
        debug_assertions,
        not(any(target_os = "android", target_os = "ios"))
    ))]
    let router = configured_router_with_segmented_debug(
        hls_sessions,
        client,
        progressive.clone(),
        delivery.clone(),
        nostr,
        segmented.clone(),
    );
    #[cfg(not(all(
        feature = "video-debug-web",
        debug_assertions,
        not(any(target_os = "android", target_os = "ios"))
    )))]
    let router = configured_router_with_segmented(
        hls_sessions,
        client,
        progressive.clone(),
        segmented.clone(),
    );
    #[cfg(not(all(
        feature = "video-debug-web",
        debug_assertions,
        not(any(target_os = "android", target_os = "ios"))
    )))]
    let _ = nostr;
    Ok((router, delivery, progressive, segmented, discovery_demand))
}

/// The store as the last run left it. Adopting its contents is what
/// makes prefetched bytes survive a restart, and it is also what lets
/// capacity pressure evict files this run never wrote.
async fn opened_store(configuration: &GatewayConfiguration) -> PartialRangeStore {
    let store = PartialRangeStore::with_capacity(
        configuration.cache_directory.join("progressive"),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(configuration.max_storage_bytes),
    );
    if let Err(error) = store.load_existing().await {
        warn!("Video store could not reload its contents: {error:#}");
    }
    store
}

fn delivery_config(
    configuration: &GatewayConfiguration,
    resources: DeliveryResources,
) -> DeliveryManagerConfig<Arc<dyn MediaHttpRequests>> {
    let params = EngineParams {
        balanced_concurrency: configuration.max_parallel_downloads,
        ..EngineParams::default()
    };
    DeliveryManagerConfig {
        store: resources.store,
        client: resources.client,
        cache: resources.cache,
        segmented: resources.segmented,
        network: resources.network,
        stats_path: configuration.cache_directory.join("host_stats.json"),
        params,
        level: DataUsageLevel::Balanced,
        tuning: DeliveryTuning::default(),
    }
}
