#![cfg(feature = "video-debug-web")]

mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::Request;
use gateway_fixture::progressive::progressive_harness;
use tower::ServiceExt;

#[tokio::test]
async fn dashboard_exposes_live_parallel_video_lanes() {
    let html = asset("/debug").await;
    assert!(html.contains("id=\"parallel-retrieval\""));
    assert!(html.contains("id=\"parallel-count\""));
    assert!(html.contains("id=\"delivery-lanes\""));
    assert!(html.contains("/debug/delivery_lanes.js"));

    let app = asset("/debug/app.js").await;
    assert!(app.contains("renderDeliveryActivity(state)"));

    let renderer = asset("/debug/delivery_lanes.js").await;
    assert!(renderer.contains("activeConnections"));
    assert!(renderer.contains("plan?.allocations"));
    assert!(renderer.contains("simultaneous"));
    assert!(renderer.contains("video.downloaded_bytes"));
}

async fn asset(path: &str) -> String {
    let harness = progressive_harness("ghostr-debug-parallel-delivery-web");
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
