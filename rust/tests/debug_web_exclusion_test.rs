#![cfg(not(feature = "video-debug-web"))]

mod support;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use support::progressive::progressive_harness;
use tower::ServiceExt;

#[tokio::test]
async fn ordinary_gateway_builds_do_not_expose_the_debug_page() {
    let harness = progressive_harness("debug-web-excluded");
    let request = Request::builder()
        .uri("/debug")
        .body(Body::empty())
        .expect("request");

    let response = harness.router.oneshot(request).await.expect("response");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
