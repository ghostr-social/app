//! The delivery control surface refuses calls until the engine runs.
//! This binary must never start the engine: the guard is the behavior.

use rust_lib_ghostr::api::delivery_types::FfiFocusItem;
use rust_lib_ghostr::api::engine_control::ffi_set_delivery_config;
use rust_lib_ghostr::api::focus_control::{ffi_playback_url, ffi_update_focus};

fn progressive_item(id: &str) -> FfiFocusItem {
    FfiFocusItem {
        post_id: id.to_owned(),
        urls: vec![format!("https://media.example/{id}.mp4")],
        delivery: "progressive".to_owned(),
        sha256: None,
        size_bytes: Some(16),
        duration_ms: Some(2_000),
    }
}

#[tokio::test]
async fn refuses_delivery_control_before_the_engine_starts() {
    let focus = ffi_update_focus("feed".to_owned(), vec![progressive_item("clip")], 0, 0).await;
    let url = ffi_playback_url(progressive_item("clip")).await;
    let config = ffi_set_delivery_config("balanced".to_owned(), 1024).await;

    assert!(focus.is_err());
    assert!(url.is_err());
    assert!(config.is_err());
}
