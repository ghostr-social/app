use super::DeliveryResources;
use crate::runtime::GatewayConfiguration;
use ghostr_delivery::manager::{DeliveryManagerConfig, DeliveryTuning};
use ghostr_engine::{DataUsageLevel, EngineParams};
use std::sync::Arc;

pub(super) fn build(
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
        transform: Some(Arc::new(
            ghostr_delivery::transform::FastStartRemuxBackend::production(),
        )),
    }
}
