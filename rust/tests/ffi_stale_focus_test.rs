//! A delayed Flutter focus write cannot replace a newer generation.

mod support;

use rust_lib_ghostr::api::delivery_types::{FfiFocusItem, FfiMediaDelivery};
use rust_lib_ghostr::api::focus_control::{ffi_update_focus, FfiFocusTransition, FfiFocusUpdate};
use support::fixtures::temp_directory;

#[tokio::test]
async fn rejects_a_focus_generation_superseded_in_transit() {
    let directory = temp_directory("ghostr-stale-focus");
    support::engine::start(&directory, 1_024)
        .await
        .expect("engine start");
    ffi_update_focus(update(2, "current"))
        .await
        .expect("current focus");

    let error = ffi_update_focus(update(1, "stale"))
        .await
        .expect_err("stale focus must be rejected");

    assert!(error.to_string().contains("superseded"));
    std::fs::remove_dir_all(directory).expect("remove cache");
}

fn update(generation: u64, post_id: &str) -> FfiFocusUpdate {
    FfiFocusUpdate {
        feed_id: "feed".to_owned(),
        items: vec![progressive_item(post_id)],
        current_index: 0,
        watch_ms: 0,
        generation,
        transition: FfiFocusTransition::UserNavigation,
    }
}

fn progressive_item(post_id: &str) -> FfiFocusItem {
    FfiFocusItem {
        post_id: post_id.to_owned(),
        urls: vec![format!("https://media.example/{post_id}.mp4")],
        delivery: FfiMediaDelivery::Progressive,
        sha256: None,
        size_bytes: Some(16),
        duration_ms: Some(2_000),
    }
}
