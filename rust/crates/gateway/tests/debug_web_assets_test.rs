#![cfg(feature = "video-debug-web")]

mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::{header::CONTENT_TYPE, Request, StatusCode};
use gateway_fixture::progressive::progressive_harness;
use tower::ServiceExt;

#[tokio::test]
async fn debug_page_serves_every_local_runtime_asset_with_its_media_type() {
    for (path, media_type) in [
        ("/debug", "text/html"),
        ("/debug/", "text/html"),
        ("/debug/app.js", "text/javascript"),
        ("/debug/player_events.js", "text/javascript"),
        ("/debug/network_modal.js", "text/javascript"),
        ("/debug/nostr_feed.js", "text/javascript"),
        ("/debug/video_form.js", "text/javascript"),
        ("/debug/hls.min.js", "text/javascript"),
        ("/debug/hls_player.js", "text/javascript"),
        ("/debug/styles.css", "text/css"),
        ("/debug/hls.LICENSE.txt", "text/plain"),
    ] {
        let (status, content_type, bytes) = asset(path).await;
        assert_eq!(status, StatusCode::OK, "{path}");
        assert!(
            content_type.starts_with(media_type),
            "{path}: {content_type}"
        );
        assert!(!bytes.is_empty(), "{path}");
    }
}

async fn asset(path: &str) -> (StatusCode, String, bytes::Bytes) {
    let harness = progressive_harness("ghostr-debug-assets");
    let request = Request::builder()
        .uri(path)
        .body(Body::empty())
        .expect("request");
    let response = harness.router.oneshot(request).await.expect("response");
    let status = response.status();
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    (status, content_type, body)
}
