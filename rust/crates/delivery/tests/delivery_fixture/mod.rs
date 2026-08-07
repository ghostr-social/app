//! Harness that runs a real delivery manager against fixture servers
//! and a temp partial-range store.

#![allow(dead_code)]

pub mod aba_origin;
pub mod full_disk;
pub mod items;
pub mod media;
pub mod options;
pub mod probe_origins;
pub mod retry;
pub mod wait;

use ghostr_delivery::debug_network::NetworkThrottle;
use ghostr_delivery::delivery_events::DeliveryHandle;
use ghostr_delivery::delivery_manager::{start_delivery_manager, DeliveryManagerConfig};
use ghostr_delivery::playback_demand::{demand_channel, DemandSender};
use ghostr_delivery::progressive_posts::ServablePosts;
use ghostr_net::outbound_media_client::MediaHttpClient;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

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

/// A directory no other caller holds. The clock alone cannot promise
/// that: it repeats a nanosecond reading often enough that two fixtures
/// built in the same instant would share a root, so the process and a
/// per-call counter carry the uniqueness and the reading only separates
/// this run from an earlier one that left a directory behind.
pub fn temp_directory(prefix: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let sequence = NEXT.fetch_add(1, Ordering::Relaxed);
    let process = std::process::id();
    std::env::temp_dir().join(format!("{prefix}-{nonce}-{process}-{sequence}"))
}

pub fn media_client() -> MediaHttpClient {
    MediaHttpClient::trusted().expect("trusted media client")
}

pub fn start_harness(prefix: &str, options: DeliveryOptions) -> DeliveryHarness {
    start_harness_at(temp_directory(prefix), options)
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
