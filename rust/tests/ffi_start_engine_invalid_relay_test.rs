mod support;

use rust_lib_ghostr::api::engine_control::{
    ffi_start_engine, FfiDataUsageLevel, FfiEngineConfiguration,
};
use rust_lib_ghostr::api::network_control::FfiDeliveryNetworkStatus;
use support::fixtures::temp_directory;

#[tokio::test]
async fn rejects_invalid_relay_urls_before_starting() {
    let directory = temp_directory("ghostr-engine-relay");
    let configuration = FfiEngineConfiguration {
        read_relay_urls: vec!["https://not-a-relay.example".to_owned()],
        search_relay_urls: Vec::new(),
        data_usage: FfiDataUsageLevel::Balanced,
        max_storage_bytes: 1024,
    };

    let result = ffi_start_engine(
        directory.to_string_lossy().to_string(),
        configuration,
        None,
        FfiDeliveryNetworkStatus::unavailable(),
    )
    .await;

    assert!(result.is_err());
    assert!(!directory.exists());
}
