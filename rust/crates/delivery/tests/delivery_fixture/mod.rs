//! Harness that runs a real delivery manager against fixture servers
//! and a temp partial-range store.

#![allow(dead_code)]

pub mod aba_origin;
pub mod concurrency_origin;
mod environment;
pub mod full_disk;
pub mod host_hol;
pub mod items;
pub mod media;
pub mod options;
pub mod paced_media;
pub mod probe_origins;
pub mod retry;
pub mod stats;
pub mod wait;

use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_delivery::delivery_events::DeliveryHandle;
use ghostr_delivery::manager::{start_delivery_manager_with_modes, DeliveryManagerConfig};
use ghostr_delivery::playback_demand::{demand_channel, DemandSender};
use ghostr_delivery::progressive_posts::ServablePosts;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub use environment::{media_client, temp_directory};
use options::DeliveryOptions;

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
    start_harness_at(temp_directory(prefix), options)
}

pub fn start_harness_at(root: PathBuf, options: DeliveryOptions) -> DeliveryHarness {
    let store = PartialRangeStore::with_capacity(
        root.clone(),
        Arc::new(Mutex::new(0)),
        StoreCapacity::system(u64::MAX),
    );
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
    let (handle, _modes) = start_delivery_manager_with_modes(
        DeliveryManagerConfig {
            store: store.clone(),
            client: media_client(),
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
