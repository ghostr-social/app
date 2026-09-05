//! A real HTTP response's no-store policy reaches the production payload sink.
mod delivery_fixture;
use axum::{http::HeaderMap, routing::get, Router};
use delivery_fixture::{
    items::{focus_now, sized_item},
    start_harness,
    wait::wait_for_ranges,
};
use sha2::{Digest as _, Sha256};

#[tokio::test]
async fn account_reset_revokes_private_playback_but_preserves_public_bytes() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("fixture");
    let url = format!(
        "http://{}/video.mp4",
        listener.local_addr().expect("fixture")
    );
    let server = tokio::spawn(async move {
        axum::serve(listener, Router::new().route("/video.mp4", get(response)))
            .await
            .expect("fixture");
    });
    let harness = start_harness("no-store-origin", Default::default());
    let mut item = sized_item("post", &url, 8, 1_000);
    item.meta.sha256 = Some(format!("{:x}", Sha256::digest(b"new data")));
    harness.handle.update_focus(focus_now(vec![item], 0, 0));
    wait_for_ranges(&harness.store, "post", &[(0, 8)]).await;
    assert_eq!(
        harness
            .store
            .read_range("post", 0..8)
            .await
            .expect("fixture"),
        Some(b"new data".to_vec())
    );
    assert_eq!(
        harness.store.used_bytes().await,
        0,
        "no-store body must never enter the disk cache"
    );
    harness
        .store
        .write_range("public", 0, b"public")
        .await
        .expect("fixture");
    harness
        .handle
        .reset_playback_access()
        .await
        .expect("fixture");
    assert_eq!(
        harness
            .store
            .read_range("post", 0..8)
            .await
            .expect("fixture"),
        None
    );
    assert!(harness
        .store
        .present_ranges("post")
        .await
        .expect("fixture")
        .is_empty());
    assert_eq!(
        harness
            .store
            .read_range("public", 0..6)
            .await
            .expect("fixture"),
        Some(b"public".to_vec())
    );
    assert!(!harness.cache.contains("post"));
    harness.handle.clear().await.expect("fixture");
    server.abort();
    tokio::fs::remove_dir_all(harness.root)
        .await
        .expect("fixture");
}

async fn response(_: HeaderMap) -> impl axum::response::IntoResponse {
    (
        [
            ("content-type", "video/mp4"),
            ("cache-control", "no-store"),
            ("etag", "\"v1\""),
        ],
        "new data",
    )
}
