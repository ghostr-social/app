//! One started engine per process: this scenario walks the whole
//! delivery control surface against a single `ffi_start_engine` call.

mod support;

use rust_lib_ghostr::api::delivery_types::{FfiFocusItem, FfiMediaDelivery};
use rust_lib_ghostr::api::engine_control::{
    ffi_set_delivery_config, ffi_start_engine, FfiDataUsageLevel, FfiEngineConfiguration,
};
use rust_lib_ghostr::api::focus_control::{
    ffi_playback_url, ffi_update_focus, FfiFocusTransition, FfiFocusUpdate,
};
use rust_lib_ghostr::api::network_control::FfiDeliveryNetworkStatus;
use rust_lib_ghostr::api::playback_control::ffi_report_playback;
use rust_lib_ghostr::api::playback_types::{FfiPlaybackObservation, FfiPlaybackPhase};
use support::fixtures::temp_directory;

const FALLBACK_ID: &str = "url-9749fdddd453caaca021690db04c6aeaa579386dd6e8fb127cd82c47a3d52f55";

fn progressive_item(id: &str) -> FfiFocusItem {
    FfiFocusItem {
        post_id: id.to_owned(),
        urls: vec![format!("https://media.example/{id}.mp4")],
        delivery: FfiMediaDelivery::Progressive,
        sha256: None,
        size_bytes: Some(16),
        duration_ms: Some(2_000),
        blurhash: None,
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
async fn starts_the_engine_and_rejects_unavailable_playback() {
    let directory = temp_directory("ghostr-engine-start");
    let endpoint = ffi_start_engine(
        directory.to_string_lossy().to_string(),
        configuration(FfiDataUsageLevel::Conservative, 1024),
        None,
        FfiDeliveryNetworkStatus::unavailable(),
    )
    .await
    .expect("engine endpoint");

    let health = reqwest::get(format!("http://{endpoint}/status"))
        .await
        .expect("health request");
    assert_eq!(health.status(), reqwest::StatusCode::NO_CONTENT);

    ffi_update_focus(FfiFocusUpdate {
        feed_id: "feed".to_owned(),
        items: vec![progressive_item(FALLBACK_ID)],
        current_index: 0,
        watch_ms: 0,
        generation: 1,
        transition: FfiFocusTransition::UserNavigation,
        rescue: None,
    })
    .await
    .expect("focus update");
    ffi_report_playback(FfiPlaybackObservation {
        post_id: FALLBACK_ID.to_owned(),
        generation: 1,
        sequence: 1,
        phase: FfiPlaybackPhase::Playing,
        position_ms: 0,
        buffered_extent_ms: 1_000,
        playback_rate_milli: 1_000,
    })
    .await
    .expect("playback report");
    ffi_set_delivery_config(configuration(FfiDataUsageLevel::Aggressive, 2048))
        .await
        .expect("config update");

    let playback = ffi_playback_url(progressive_item(FALLBACK_ID)).await;
    assert!(playback.is_err(), "unavailable media is not player-ready");

    let rejected = ffi_set_delivery_config(configuration(FfiDataUsageLevel::Balanced, 0)).await;
    assert!(rejected.is_err());
    std::fs::remove_dir_all(directory).expect("remove cache");
}
