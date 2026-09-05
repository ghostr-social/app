#![cfg(all(feature = "device-integration", debug_assertions))]
//! Logout revokes issued media URLs at the real FFI/gateway boundary.
mod support;
use axum::{routing::get, Router};
use rust_lib_ghostr::api::delivery_types::{FfiFocusItem, FfiMediaDelivery};
use rust_lib_ghostr::api::focus_control::{
    ffi_playback_url, ffi_update_focus, FfiFocusTransition, FfiFocusUpdate,
};
use rust_lib_ghostr::api::session_control::ffi_reset_nostr_session;

#[tokio::test]
async fn reset_revokes_an_issued_private_playback_url() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("fixture");
    let origin = format!("http://{}", listener.local_addr().expect("fixture"));
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            Router::new().route(
                "/video.mp4",
                get(|| async {
                    (
                        [("cache-control", "no-store"), ("content-type", "video/mp4")],
                        "private!",
                    )
                }),
            ),
        )
        .await
        .expect("fixture");
    });
    let root = support::fixtures::temp_directory("session-media-reset");
    support::engine::start_with_device_origin(&root, 1_048_576, origin.clone())
        .await
        .expect("fixture");
    let item = FfiFocusItem {
        post_id: "private-post".to_owned(),
        urls: vec![format!("{origin}/video.mp4")],
        delivery: FfiMediaDelivery::Progressive,
        sha256: None,
        size_bytes: Some(8),
        duration_ms: Some(1_000),
        blurhash: None,
    };
    ffi_update_focus(FfiFocusUpdate {
        feed_id: "feed".to_owned(),
        items: vec![item.clone()],
        current_index: 0,
        watch_ms: 0,
        generation: 1,
        transition: FfiFocusTransition::UserNavigation,
        rescue: None,
    })
    .await
    .expect("fixture");
    let url = ffi_playback_url(item).await.expect("fixture");
    let client = reqwest::Client::new();
    let before = client.get(&url).send().await.expect("fixture");
    assert!(before.status().is_success());
    assert_eq!(before.bytes().await.expect("fixture").as_ref(), b"private!");
    ffi_reset_nostr_session(None).await.expect("fixture");
    let after = client.get(&url).send().await.expect("fixture");
    assert!(
        after.status().is_client_error(),
        "old URL remains authorized: {}",
        after.status()
    );
    server.abort();
    std::fs::remove_dir_all(root).expect("fixture");
}
