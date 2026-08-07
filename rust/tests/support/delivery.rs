//! Harness that runs a real delivery manager against fixture servers
//! and a temp partial-range store.

use rust_lib_ghostr::video::debug_network::NetworkThrottle;
use rust_lib_ghostr::video::delivery_events::DeliveryHandle;
use rust_lib_ghostr::video::delivery_manager::{start_delivery_manager, DeliveryManagerConfig};
use rust_lib_ghostr::video::outbound_media_client::MediaHttpClient;
use rust_lib_ghostr::video::partial_range_store::PartialRangeStore;
use rust_lib_ghostr::video::playback_demand::{demand_channel, DemandSender};
use rust_lib_ghostr::video::progressive_posts::ServablePosts;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::delivery_options::DeliveryOptions;

pub struct DeliveryHarness {
    pub handle: DeliveryHandle,
    pub demand: DemandSender,
    pub store: Arc<PartialRangeStore>,
    pub posts: ServablePosts,
    pub cache: ServablePosts,
    pub network: NetworkThrottle,
    pub root: PathBuf,
}

pub fn start_harness(prefix: &str, options: DeliveryOptions) -> DeliveryHarness {
    start_harness_at(super::fixtures::temp_directory(prefix), options)
}

pub fn start_harness_at(root: PathBuf, options: DeliveryOptions) -> DeliveryHarness {
    let store = PartialRangeStore::new(root.clone(), Arc::new(Mutex::new(0)));
    start_harness_with_store(Arc::new(store), root, options)
}

/// A manager over a store the test built itself, e.g. one whose free
/// space it can move.
pub fn start_harness_with_store(
    store: Arc<PartialRangeStore>,
    root: PathBuf,
    options: DeliveryOptions,
) -> DeliveryHarness {
    let posts = ServablePosts::new();
    let network = NetworkThrottle::new();
    let (demand, demand_receiver) = demand_channel();
    let handle = start_delivery_manager(
        DeliveryManagerConfig {
            store: store.clone(),
            client: MediaHttpClient::trusted().expect("trusted media client"),
            cache: posts.clone(),
            network: network.clone(),
            stats_path: root.join("host_stats.json"),
            params: options.params,
            level: options.level,
            tuning: options.tuning,
        },
        demand_receiver,
    );
    let cache = posts.clone();
    DeliveryHarness {
        handle,
        demand,
        store,
        posts,
        cache,
        network,
        root,
    }
}
