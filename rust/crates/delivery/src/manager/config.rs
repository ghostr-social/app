use crate::cache_registry::CacheRegistry;
use crate::debug::network::NetworkThrottle;
use crate::segmented::SegmentedCache;
use ghostr_engine::{DataUsageLevel, EngineParams};
use ghostr_net::media_request_executor::MediaRequestExecutor;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::PathBuf;
use std::sync::Arc;

/// Everything the manager owns or reaches, as one typed object.
pub struct DeliveryManagerConfig {
    pub store: Arc<PartialRangeStore>,
    pub requests: MediaRequestExecutor,
    pub cache: CacheRegistry,
    pub segmented: SegmentedCache,
    pub network: NetworkThrottle,
    pub network_status: crate::delivery_events::DeliveryNetworkStatus,
    pub stats_path: PathBuf,
    pub params: EngineParams,
    pub level: DataUsageLevel,
    pub tuning: super::DeliveryTuning,
    pub transform: Option<Arc<dyn crate::transform::TransformBackend>>,
}
