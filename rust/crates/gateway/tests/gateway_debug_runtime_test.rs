#![cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]

mod gateway_fixture;

use ghostr_discovery::cache::client_with_event_cache;
use ghostr_gateway::runtime::{GatewayConfiguration, GatewayRuntime};
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::test]
async fn debug_runtime_starts_on_a_loopback_endpoint() {
    let root = gateway_fixture::temp_directory("debug-runtime");
    let (endpoint, _runtime, _modes) = GatewayRuntime::start_debug(
        GatewayConfiguration {
            cache_directory: root.clone(),
            relays: Vec::new(),
            max_parallel_downloads: 1,
            max_storage_bytes: 1_024,
            device_integration_origin: None,
        },
        Arc::new(client_with_event_cache()),
    )
    .await
    .expect("debug runtime");

    let address: SocketAddr = endpoint.parse().expect("socket address");
    assert!(address.ip().is_loopback());
    std::fs::remove_dir_all(root).ok();
}
