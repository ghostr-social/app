use super::progressive::ProgressiveHarness;
use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use serde_json::Value;
use tower::ServiceExt;

pub async fn json_request(harness: &ProgressiveHarness, request: Request<Body>) -> Value {
    let response = harness
        .router
        .clone()
        .oneshot(request)
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&bytes).expect("json")
}
