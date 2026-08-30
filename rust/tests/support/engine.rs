use rust_lib_ghostr::api::engine_control::{
    ffi_start_engine, FfiDataUsageLevel, FfiEngineConfiguration,
};
use rust_lib_ghostr::api::network_control::FfiDeliveryNetworkStatus;
use std::path::Path;

pub fn configuration(max_storage_bytes: u64) -> FfiEngineConfiguration {
    FfiEngineConfiguration {
        read_relay_urls: Vec::new(),
        search_relay_urls: Vec::new(),
        data_usage: FfiDataUsageLevel::Balanced,
        max_storage_bytes,
    }
}

pub async fn start(directory: &Path, max_storage_bytes: u64) -> anyhow::Result<String> {
    ffi_start_engine(
        directory.to_string_lossy().to_string(),
        configuration(max_storage_bytes),
        None,
        FfiDeliveryNetworkStatus::unavailable(),
    )
    .await
}

pub async fn start_with_device_origin(
    directory: &Path,
    max_storage_bytes: u64,
    origin: String,
) -> anyhow::Result<String> {
    ffi_start_engine(
        directory.to_string_lossy().to_string(),
        configuration(max_storage_bytes),
        Some(origin),
        FfiDeliveryNetworkStatus::wifi(1),
    )
    .await
}
