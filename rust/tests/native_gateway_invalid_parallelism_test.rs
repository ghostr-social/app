mod support;

use ghostr_discovery::cache::client_with_event_cache;
use ghostr_gateway::runtime::{GatewayConfiguration, GatewayRuntime};
use std::sync::Arc;
use support::fixtures::temp_directory;

#[tokio::test]
async fn rejects_zero_native_download_parallelism() {
    let directory = temp_directory("ghostr-gateway-parallelism");

    let result = GatewayRuntime::start(
        GatewayConfiguration {
            cache_directory: directory.clone(),
            relays: Vec::new(),
            max_parallel_downloads: 0,
            max_storage_bytes: 1024,
            internet_data_limit: ghostr_net::internet_allowance::InternetDataLimit::Unlimited,
            network_status: ghostr_delivery::delivery_events::DeliveryNetworkStatus::unavailable(),
            device_integration_origin: None,
        },
        Arc::new(client_with_event_cache()),
    )
    .await;

    assert!(result.is_err());
    assert!(!directory.exists());
}
