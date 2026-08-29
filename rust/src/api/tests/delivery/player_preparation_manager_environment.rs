use ghostr_delivery::cache_registry::CacheRegistry;
use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_delivery::delivery_events::{DeliveryHandle, DeliveryNetworkStatus};
use ghostr_delivery::manager::{
    start_delivery_manager_with_discovery_demand, DeliveryManagerConfig, DeliveryTuning,
};
use ghostr_delivery::playback_demand::{demand_channel, DemandSender};
use ghostr_delivery::segmented::SegmentedCache;
use ghostr_engine::adaptive::DiscoveryDemand;
use ghostr_engine::{DataUsageLevel, EngineParams};
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::outbound_media_client::MediaHttpRequests;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::watch;

pub(super) fn start(
    store: Arc<PartialRangeStore>,
    cache: CacheRegistry,
) -> (
    DeliveryHandle,
    DemandSender,
    watch::Receiver<DiscoveryDemand>,
    PathBuf,
) {
    let root = unique_stats_root();
    let (demand, demand_receiver) = demand_channel();
    let config = manager_config(store, cache, root.join("host_stats.json"));
    let (delivery, discovery) =
        start_delivery_manager_with_discovery_demand(config, demand_receiver);
    (delivery, demand, discovery, root)
}

fn manager_config(
    store: Arc<PartialRangeStore>,
    cache: CacheRegistry,
    stats_path: PathBuf,
) -> DeliveryManagerConfig {
    DeliveryManagerConfig {
        store,
        requests: MediaRequestExecutor::new(
            Arc::new(NoRequest),
            MediaRequestLimits::try_new(1, 1).expect("test fixture precondition must hold"),
        ),
        cache,
        segmented: SegmentedCache::new(),
        network: NetworkThrottle::new(),
        network_status: DeliveryNetworkStatus::unavailable(),
        stats_path,
        params: EngineParams::default(),
        level: DataUsageLevel::Balanced,
        tuning: DeliveryTuning::default(),
        transform: None,
    }
}

fn unique_stats_root() -> PathBuf {
    static NEXT_ROOT: AtomicU64 = AtomicU64::new(1);
    let root = std::env::temp_dir().join(format!(
        "ghostr-player-preparation-manager-{}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("test fixture precondition must hold")
            .as_nanos(),
        NEXT_ROOT.fetch_add(1, Ordering::Relaxed),
    ));
    std::fs::create_dir_all(&root).expect("test fixture precondition must hold");
    root
}

struct NoRequest;

impl MediaHttpRequests for NoRequest {
    fn get(&self, _url: &str) -> anyhow::Result<reqwest::RequestBuilder> {
        anyhow::bail!("completed player-preparation fixture must not use the network")
    }
}
