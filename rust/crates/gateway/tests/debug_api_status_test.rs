#![cfg(feature = "video-debug-web")]

mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::{header::CONTENT_TYPE, Method, Request, StatusCode};
use gateway_fixture::progressive::progressive_harness;
use ghostr_engine::{DeliveryKind, VideoMeta};
use serde_json::{json, Value};
use tower::ServiceExt;

fn metadata() -> VideoMeta {
    VideoMeta {
        urls: vec!["https://relay.example/video.mp4".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(100),
        duration_ms: Some(60_000),
    }
}

#[tokio::test]
async fn debug_state_distinguishes_queued_and_cached_videos() {
    let harness = progressive_harness("ghostr-debug-status");
    harness.posts.insert_video("cached", metadata());
    harness.posts.insert_video("queued", metadata());
    harness
        .store
        .set_total_len("cached", 100)
        .await
        .expect("total");
    harness
        .store
        .set_total_len("queued", 100)
        .await
        .expect("total");
    harness
        .store
        .write_range("cached", 0, &[1; 100])
        .await
        .expect("write");
    let request = Request::builder()
        .uri("/debug/api/state")
        .body(Body::empty())
        .expect("request");
    let response = harness.router.oneshot(request).await.expect("response");
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let state: Value = serde_json::from_slice(&body).expect("state");

    assert_eq!(state["videos"][0]["status"], "cached");
    assert_eq!(state["videos"][0]["downloaded_duration_ms"], 60_000);
    assert_eq!(state["videos"][1]["status"], "queued");
    assert_eq!(state["videos"][1]["downloaded_duration_ms"], 0);
}

#[tokio::test]
async fn debug_network_rejects_conditions_outside_safe_bounds() {
    let harness = progressive_harness("ghostr-debug-invalid-network");
    let request = Request::builder()
        .method(Method::PUT)
        .uri("/debug/api/network")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({"bandwidth_kbps": 0, "latency_ms": 60001,
            "max_connections_per_host": 0})
            .to_string(),
        ))
        .expect("request");

    let response = harness.router.oneshot(request).await.expect("response");

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
