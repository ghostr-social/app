#![cfg(feature = "video-debug-web")]

mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use gateway_fixture::progressive::progressive_harness;
use tower::ServiceExt;

#[tokio::test]
async fn dashboard_embeds_portable_hls_playback_without_a_runtime_cdn() {
    let html = asset("/debug").await;
    assert!(html.contains("/debug/hls.min.js"));
    assert!(html.contains("/debug/hls_player.js"));
    assert!(!html.contains("<script src=\"https://"));

    let library = asset("/debug/hls.min.js").await;
    assert!(library.contains("hls.js"));
    assert!(library.contains("1.6.16"));
    assert!(asset("/debug/hls.LICENSE.txt")
        .await
        .contains("Apache License"));

    let player = asset("/debug/hls_player.js").await;
    assert!(player.contains("/debug/api/hls"));
    assert!(player.contains("method: \"POST\""));
    assert!(player.contains("method: \"DELETE\""));
    assert!(player.contains("Hls.isSupported()"));
    assert!(player.contains("canPlayType"));
    assert!(player.contains(".destroy()"));

    let app = asset("/debug/app.js").await;
    assert!(app.contains("startPlayback(video"));
    assert!(app.contains(
        "function hydratePlayback(state) {\n  if (currentId) return;\n  const videos = debugVideos(state);"
    ));
}

async fn asset(path: &str) -> String {
    let harness = progressive_harness("debug-web-hls-assets");
    let response = harness
        .router
        .oneshot(
            Request::builder()
                .uri(path)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    String::from_utf8(body.to_vec()).expect("text")
}
