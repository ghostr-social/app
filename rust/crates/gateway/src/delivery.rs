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
use crate::router::GatewayRouterResources;
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
use ghostr_net::media_request_executor::MediaRequestExecutor;
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
    requests: MediaRequestExecutor,
    cache: CacheRegistry,
    network: NetworkThrottle,
    segmented: SegmentedCache,
}

struct RouterInput {
    hls_sessions: HlsSessions,
    requests: MediaRequestExecutor,
    segmented: SegmentedCache,
    progressive: Arc<ProgressiveState>,
    delivery: DeliveryHandle,
    nostr: Arc<Client>,
}

impl DeliveryResources {
    async fn open(configuration: &GatewayConfiguration, requests: MediaRequestExecutor) -> Self {
        Self {
            store: Arc::new(opened_store(configuration).await),
            requests,
            cache: CacheRegistry::new(),
            network: NetworkThrottle::new(),
            segmented: SegmentedCache::new(),
        }
    }
}

/// Progressive delivery: the router serves `/video.mp4` from the partial
/// store; adaptive candidate demand feeds the discovery control loop.
pub(crate) async fn start_progressive_delivery(
    configuration: &GatewayConfiguration,
    hls_sessions: HlsSessions,
    nostr: Arc<Client>,
    requests: MediaRequestExecutor,
) -> anyhow::Result<DeliveryParts> {
    let resources = DeliveryResources::open(configuration, requests).await;
    let (demand_sender, demand) = demand_channel();
    let config = delivery_config(configuration, &resources);
    let (delivery, discovery_demand) = start_delivery_manager_with_discovery_demand(config, demand);
    let progressive = progressive_state(configuration, &resources, &delivery, demand_sender);
    let router = delivery_router(RouterInput {
        hls_sessions,
        requests: resources.requests.clone(),
        segmented: resources.segmented.clone(),
        progressive: progressive.clone(),
        delivery: delivery.clone(),
        nostr,
    });
    Ok((
        router,
        delivery,
        progressive,
        resources.segmented,
        discovery_demand,
    ))
}

fn progressive_state(
    configuration: &GatewayConfiguration,
    resources: &DeliveryResources,
    delivery: &DeliveryHandle,
    demand: ghostr_delivery::playback_demand::DemandSender,
) -> Arc<ProgressiveState> {
    Arc::new(ProgressiveState {
        store: resources.store.clone(),
        demand,
        cache: resources.cache.clone(),
        network: resources.network.clone(),
        timing: ProgressiveTiming::default(),
        capabilities: ProgressiveCapabilities::production(),
        #[cfg(all(
            feature = "video-debug-web",
            debug_assertions,
            not(any(target_os = "android", target_os = "ios"))
        ))]
        debug_feed: DebugFeed::new(delivery.clone(), configuration.relays.clone()),
    })
}

fn delivery_router(input: RouterInput) -> axum::Router {
    let router_resources = GatewayRouterResources::new(input.hls_sessions, input.requests)
        .with_segmented(input.segmented);
    #[cfg(all(
        feature = "video-debug-web",
        debug_assertions,
        not(any(target_os = "android", target_os = "ios"))
    ))]
    let router = configured_router_with_segmented_debug(
        router_resources,
        input.progressive,
        input.delivery,
        input.nostr,
    );
    #[cfg(not(all(
        feature = "video-debug-web",
        debug_assertions,
        not(any(target_os = "android", target_os = "ios"))
    )))]
    let router = configured_router_with_segmented(router_resources, input.progressive);
    #[cfg(not(all(
        feature = "video-debug-web",
        debug_assertions,
        not(any(target_os = "android", target_os = "ios"))
    )))]
    let _ = (input.delivery, input.nostr);
    router
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
    resources: &DeliveryResources,
) -> DeliveryManagerConfig {
    let params = EngineParams {
        balanced_concurrency: configuration.max_parallel_downloads,
        ..EngineParams::default()
    };
    DeliveryManagerConfig {
        store: resources.store.clone(),
        requests: resources.requests.clone(),
        cache: resources.cache.clone(),
        segmented: resources.segmented.clone(),
        network: resources.network.clone(),
        stats_path: configuration.cache_directory.join("host_stats.json"),
        params,
        level: DataUsageLevel::Balanced,
        tuning: DeliveryTuning::default(),
    }
}
