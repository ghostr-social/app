//! One started engine per process: this scenario walks the whole
//! delivery control surface against a single ffi_start_engine call.

mod support;

use rust_lib_ghostr::api::delivery_types::{FfiFocusItem, FfiMediaDelivery};
use rust_lib_ghostr::api::engine_control::{
    ffi_set_delivery_config, ffi_start_engine, FfiDataUsageLevel, FfiEngineConfiguration,
};
use rust_lib_ghostr::api::focus_control::{ffi_playback_url, ffi_update_focus};
use support::fixtures::temp_directory;

fn progressive_item(id: &str) -> FfiFocusItem {
    FfiFocusItem {
        post_id: id.to_owned(),
        urls: vec![format!("https://media.example/{id}.mp4")],
        delivery: FfiMediaDelivery::Progressive,
        sha256: None,
        size_bytes: Some(16),
        duration_ms: Some(2_000),
    }
}

fn configuration(data_usage: FfiDataUsageLevel, max_storage_bytes: u64) -> FfiEngineConfiguration {
    FfiEngineConfiguration {
        read_relay_urls: Vec::new(),
        search_relay_urls: Vec::new(),
        data_usage,
        max_storage_bytes,
    }
}

#[tokio::test]
async fn starts_the_engine_and_serves_the_delivery_surface() {
    let directory = temp_directory("ghostr-engine-start");
    let endpoint = ffi_start_engine(
        directory.to_string_lossy().to_string(),
        configuration(FfiDataUsageLevel::Conservative, 1024),
    )
    .await
    .expect("engine endpoint");

    let health = reqwest::get(format!("http://{endpoint}/status"))
        .await
        .expect("health request");
    assert_eq!(health.status(), reqwest::StatusCode::NO_CONTENT);

    ffi_update_focus("feed".to_owned(), vec![progressive_item("clip")], 0, 0)
        .await
        .expect("focus update");
    ffi_set_delivery_config(configuration(FfiDataUsageLevel::Aggressive, 2048))
        .await
        .expect("config update");

    let url = ffi_playback_url(progressive_item("clip"))
        .await
        .expect("playback url");
    assert_eq!(url, format!("http://{endpoint}/video.mp4?id=clip"));

    let rejected = ffi_set_delivery_config(configuration(FfiDataUsageLevel::Balanced, 0)).await;
    assert!(rejected.is_err());
    std::fs::remove_dir_all(directory).expect("remove cache");
}
