use super::DeliveryResources;
use crate::runtime::GatewayConfiguration;
use ghostr_delivery::manager::{DeliveryManagerConfig, DeliveryTuning};
use ghostr_delivery::transform::{
    thread_cpu_measurement_available, FastStartRemuxBackend, TransformBackend,
};
use ghostr_engine::{DataUsageLevel, EngineParams};
use std::sync::Arc;

#[cfg(test)]
#[path = "config/transform_composition_test.rs"]
mod transform_composition_test;

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
        network_status: configuration.network_status,
        stats_path: configuration.cache_directory.join("host_stats.json"),
        params,
        level: DataUsageLevel::Balanced,
        tuning: DeliveryTuning::default(),
        transform: production_transform(thread_cpu_measurement_available()),
    }
}

fn production_transform(measurable: bool) -> Option<Arc<dyn TransformBackend>> {
    measurable.then(|| Arc::new(FastStartRemuxBackend::production()) as Arc<dyn TransformBackend>)
}
