mod support;

use rust_lib_ghostr::video::gateway_runtime::{GatewayConfiguration, GatewayRuntime};
use support::fixtures::temp_directory;

#[tokio::test]
async fn rejects_zero_native_download_parallelism() {
    let directory = temp_directory("ghostr-gateway-parallelism");

    let result = GatewayRuntime::start(GatewayConfiguration {
        cache_directory: directory.clone(),
        relays: Vec::new(),
        max_parallel_downloads: 0,
        max_storage_bytes: 1024,
    })
    .await;

    assert!(result.is_err());
    assert!(!directory.exists());
}
