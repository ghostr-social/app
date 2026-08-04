//! Wiring the progressive delivery half of the gateway: the range store
//! the router serves from, the downloader that fills it, and the
//! manager that decides what to fetch.

use crate::engine::inventory_controller::Mode;
use crate::engine::{DataUsageLevel, EngineParams};
use crate::video::delivery_events::DeliveryHandle;
use crate::video::delivery_manager::{
    start_delivery_manager_with_modes, DeliveryManagerConfig, DeliveryTuning,
};
use crate::video::gateway_runtime::GatewayConfiguration;
use crate::video::hls_sessions::HlsSessions;
use crate::video::http_gateway::configured_router_with_progressive;
use crate::video::native_models::new_native_downloads;
use crate::video::outbound_media_client::MediaHttpClient;
use crate::video::partial_range_store::capacity::StoreCapacity;
use crate::video::partial_range_store::PartialRangeStore;
use crate::video::playback_demand::demand_channel;
use crate::video::progressive_posts::ServablePosts;
use crate::video::progressive_route::{ProgressiveState, ProgressiveTiming};
use log::warn;
use std::sync::Arc;
use tokio::sync::{watch, Mutex};

pub(crate) type DeliveryParts = (
    axum::Router,
    DeliveryHandle,
    Arc<ProgressiveState>,
    watch::Receiver<Mode>,
);

/// Progressive delivery: the router serves `/video.mp4` from the partial
/// store; the manager's mode watch feeds the discovery control loop.
pub(crate) async fn start_progressive_delivery(
    configuration: &GatewayConfiguration,
    hls_sessions: HlsSessions,
) -> anyhow::Result<DeliveryParts> {
    let store = Arc::new(opened_store(configuration).await);
    let posts = ServablePosts::new();
    let (demand_sender, demand) = demand_channel();
    let client = MediaHttpClient::public()?;
    let progressive = Arc::new(ProgressiveState {
        store: store.clone(),
        demand: demand_sender,
        posts: posts.clone(),
        timing: ProgressiveTiming::default(),
    });
    let router = configured_router_with_progressive(
        new_native_downloads(), hls_sessions, client.clone(), progressive.clone(),
    );
    let config = delivery_config(configuration, store, client, posts);
    let (delivery, modes) = start_delivery_manager_with_modes(config, demand);
    Ok((router, delivery, progressive, modes))
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
    store: Arc<PartialRangeStore>,
    client: MediaHttpClient,
    posts: ServablePosts,
) -> DeliveryManagerConfig {
    let params = EngineParams {
        balanced_concurrency: configuration.max_parallel_downloads,
        ..EngineParams::default()
    };
    DeliveryManagerConfig {
        store,
        client,
        posts,
        stats_path: configuration.cache_directory.join("host_stats.json"),
        params,
        level: DataUsageLevel::Balanced,
        tuning: DeliveryTuning::default(),
    }
}
