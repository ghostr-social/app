//! Harness that runs a real delivery manager against fixture servers
//! and a temp partial-range store.

use rust_lib_ghostr::engine::{DataUsageLevel, EngineParams};
use rust_lib_ghostr::video::delivery_events::DeliveryHandle;
use rust_lib_ghostr::video::delivery_manager::{
    start_delivery_manager, DeliveryManagerConfig, DeliveryTuning,
};
use rust_lib_ghostr::video::outbound_media_client::MediaHttpClient;
use rust_lib_ghostr::video::partial_range_store::PartialRangeStore;
use rust_lib_ghostr::video::playback_demand::{demand_channel, DemandSender};
use rust_lib_ghostr::video::progressive_posts::ServablePosts;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

pub struct DeliveryHarness {
    pub handle: DeliveryHandle,
    pub demand: DemandSender,
    pub store: Arc<PartialRangeStore>,
    pub posts: ServablePosts,
    pub root: PathBuf,
}

pub struct DeliveryOptions {
    pub params: EngineParams,
    pub level: DataUsageLevel,
    pub tuning: DeliveryTuning,
}

impl Default for DeliveryOptions {
    fn default() -> Self {
        Self {
            params: base_params(),
            level: DataUsageLevel::Balanced,
            tuning: test_tuning(),
        }
    }
}

/// Small chunks and a tiny assumed bitrate so 16-byte fixture files
/// exercise real head/tail splits.
pub fn base_params() -> EngineParams {
    EngineParams {
        chunk_bytes: 8,
        assumed_bitrate_bps: 64,
        ..EngineParams::default()
    }
}

pub fn test_tuning() -> DeliveryTuning {
    DeliveryTuning {
        probe_concurrency: 2,
        failure_cooldown: Duration::from_millis(200),
        stats_debounce: Duration::ZERO,
    }
}

pub fn start_harness(prefix: &str, options: DeliveryOptions) -> DeliveryHarness {
    start_harness_at(super::fixtures::temp_directory(prefix), options)
}

pub fn start_harness_at(root: PathBuf, options: DeliveryOptions) -> DeliveryHarness {
    let store = Arc::new(PartialRangeStore::new(root.clone(), Arc::new(Mutex::new(0))));
    let posts = ServablePosts::new();
    let (demand, demand_receiver) = demand_channel();
    let config = DeliveryManagerConfig {
        store: store.clone(),
        client: MediaHttpClient::trusted().expect("trusted media client"),
        posts: posts.clone(),
        stats_path: root.join("host_stats.json"),
        params: options.params,
        level: options.level,
        tuning: options.tuning,
    };
    let handle = start_delivery_manager(config, demand_receiver);
    DeliveryHarness {
        handle,
        demand,
        store,
        posts,
        root,
    }
}
