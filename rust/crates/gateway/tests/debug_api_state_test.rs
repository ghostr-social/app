#![cfg(feature = "video-debug-web")]

mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::{header::CONTENT_TYPE, Method, Request, StatusCode};
use gateway_fixture::progressive::progressive_harness;
use ghostr_delivery::delivery_events::DeliveryCommand;
use ghostr_engine::{DeliveryKind, VideoMeta};
use serde_json::{json, Value};
use tower::ServiceExt;

async fn json_request(
    harness: &gateway_fixture::progressive::ProgressiveHarness,
    request: Request<Body>,
) -> Value {
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

#[tokio::test]
async fn debug_state_reports_downloaded_bytes_duration_and_source() {
    let harness = progressive_harness("ghostr-debug-state");
    let meta = VideoMeta {
        urls: vec!["https://relay.example/media/clip.mp4".to_owned()],
        delivery: DeliveryKind::Progressive,
        sha256: None,
        size_bytes: Some(1_000),
        duration_ms: Some(120_000),
    };
    harness
        .posts
        .replace([gateway_fixture::cache_video("clip", meta.clone())]);
    harness.bind_video_meta("clip", meta).await;
    harness
        .store
        .set_total_len("clip", 1_000)
        .await
        .expect("total");
    harness
        .store
        .write_range("clip", 0, &[7; 250])
        .await
        .expect("write");
    let request = Request::builder()
        .uri("/debug/api/state")
        .body(Body::empty())
        .expect("request");

    let state = json_request(&harness, request).await;

    assert_eq!(state["storage"]["used_bytes"], 250);
    assert_eq!(state["videos"][0]["id"], "clip");
    assert_eq!(state["videos"][0]["source_host"], "relay.example");
    assert_eq!(state["videos"][0]["downloaded_bytes"], 250);
    assert_eq!(state["videos"][0]["duration_ms"], 120_000);
    assert_eq!(state["videos"][0]["downloaded_duration_ms"], 30_000);
    let playback_url = state["videos"][0]["playback_url"]
        .as_str()
        .expect("playback URL");
    assert!(playback_url.starts_with("/video.mp4?id=clip&cap="));
}

#[tokio::test]
async fn debug_network_profile_can_be_changed_without_restart() {
    let mut harness = progressive_harness("ghostr-debug-network");
    let payload = json!({
        "bandwidth_kbps": 768,
        "latency_ms": 350,
        "packet_loss_bps": 2_500,
        "max_connections_per_host": 1
    });
    let request = Request::builder()
        .method(Method::PUT)
        .uri("/debug/api/network")
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(payload.to_string()))
        .expect("request");
    let updated = json_request(&harness, request).await;

    assert_eq!(updated, payload);
    assert_eq!(harness.network.profile().bandwidth_kbps, 768);
    assert_eq!(harness.network.profile().packet_loss_bps, 2_500);
    assert!(matches!(
        harness.debug_commands.try_control(),
        Some(DeliveryCommand::NetworkProfile { generation: 1, .. })
    ));
}
