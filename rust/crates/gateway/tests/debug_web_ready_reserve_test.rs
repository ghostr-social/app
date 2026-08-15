#![cfg(feature = "video-debug-web")]

mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::Request;
use gateway_fixture::progressive::progressive_harness;
use tower::ServiceExt;

#[tokio::test]
async fn dashboard_renders_live_ready_reserve_evidence() {
    let html = asset("/debug").await;
    assert!(html.contains("id=\"ready-reserve\""));
    assert!(html.contains("aria-labelledby=\"ready-reserve-heading\""));
    assert!(html.contains("id=\"reserve-mode\""));
    assert!(html.contains("id=\"reserve-candidates\""));
    assert!(html.contains("/debug/ready_reserve.js"));

    let app = asset("/debug/app.js").await;
    assert!(app.contains("renderReadyReserve(state.adaptive_plans)"));

    let renderer = asset("/debug/ready_reserve.js").await;
    assert!(renderer.contains("plans.at(-1)"));
    assert!(renderer.contains("ready_coverage_ms"));
    assert!(renderer.contains("underflow_risk_bps"));
    assert!(renderer.contains("candidate.status"));
}

async fn asset(path: &str) -> String {
    let harness = progressive_harness("ghostr-debug-ready-reserve-web");
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
