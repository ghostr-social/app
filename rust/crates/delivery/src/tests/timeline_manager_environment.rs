use crate::cache_registry::CacheRegistry;
use crate::debug::network::NetworkThrottle;
use crate::manager::{DeliveryManagerConfig, DeliveryTuning};
use crate::segmented::SegmentedCache;
use ghostr_engine::{DataUsageLevel, EngineParams};
use ghostr_net::media_request_executor::{MediaRequestExecutor, MediaRequestLimits};
use ghostr_net::outbound_media_client::MediaHttpRequests;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::PathBuf;
use std::sync::Arc;

pub(crate) fn config(store: Arc<PartialRangeStore>, root: PathBuf) -> DeliveryManagerConfig {
    DeliveryManagerConfig {
        store,
        requests: MediaRequestExecutor::new(
            Arc::new(NoRequest),
            MediaRequestLimits::try_new(1, 1).unwrap(),
        ),
        cache: CacheRegistry::new(),
        segmented: SegmentedCache::new(),
        network: NetworkThrottle::new(),
        network_status: crate::delivery_events::DeliveryNetworkStatus::unavailable(),
        stats_path: root.join("stats.json"),
        params: EngineParams::default(),
        level: DataUsageLevel::Balanced,
        tuning: DeliveryTuning::default(),
        transform: None,
    }
}

pub(crate) struct NoRequest;

impl MediaHttpRequests for NoRequest {
    fn get(&self, _url: &str) -> anyhow::Result<reqwest::RequestBuilder> {
        anyhow::bail!("completed fixture must not use the network")
    }
}
