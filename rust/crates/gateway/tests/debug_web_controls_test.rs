#![cfg(feature = "video-debug-web")]

mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::Request;
use gateway_fixture::progressive::progressive_harness;
use tower::ServiceExt;

async fn asset(path: &str) -> String {
    let harness = progressive_harness("ghostr-debug-controls");
    let request = Request::builder()
        .uri(path)
        .body(Body::empty())
        .expect("request");
    let response = harness.router.oneshot(request).await.expect("response");
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    String::from_utf8(body.to_vec()).expect("text asset")
}

#[tokio::test]
async fn dashboard_exposes_accessible_navigation_and_clear_actions() {
    let html = asset("/debug").await;
    assert!(html.contains("id=\"previous-video\""));
    assert!(html.contains("aria-label=\"Play previous video\""));
    assert!(html.contains("id=\"next-video\""));
    assert!(html.contains("aria-label=\"Play next video\""));
    assert!(html.contains("id=\"clear-debug-data\""));
    assert!(html.contains("Clear debug data"));
    assert!(html.contains("/debug/navigation.js"));
    assert!(html.contains("/debug/clear_data.js"));

    let navigation = asset("/debug/navigation.js").await;
    assert!(navigation.contains("playAdjacent(-1)"));
    assert!(navigation.contains("playAdjacent(1)"));
    assert!(navigation.contains("previousVideo.disabled"));
    assert!(navigation.contains("nextVideo.disabled"));

    let clear = asset("/debug/clear_data.js").await;
    assert!(clear.contains("/debug/api/data"));
    assert!(clear.contains("method: \"DELETE\""));
    assert!(clear.contains("window.confirm"));
    assert!(clear.contains("releaseHlsPlayback"));
}
