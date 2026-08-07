#![cfg(feature = "video-debug-web")]

mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use ghostr_delivery::delivery_events::{command_channel, DeliveryCommand};
use ghostr_discovery::cache::client_with_event_cache;
use ghostr_engine::DeliveryKind;
use ghostr_gateway::hls::sessions::HlsSessions;
use ghostr_gateway::router::configured_router_with_progressive_debug;
use ghostr_gateway::progressive::route::ProgressiveState;
use ghostr_media_model::native_models::new_native_downloads;
use serde_json::Value;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn debug_api_registers_a_video_with_the_rust_delivery_engine() {
    let harness = gateway_fixture::progressive::progressive_harness("debug-video-add");
    let (delivery, mut commands) = command_channel();
    let router = configured_router_with_progressive_debug(
        new_native_downloads(),
        HlsSessions::production(),
        gateway_fixture::media_client(),
        progressive_state(&harness),
        delivery,
        Arc::new(client_with_event_cache()),
    );
    let request = Request::builder()
        .method("POST")
        .uri("/debug/api/videos")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"url":"https://cdn.example/video.mp4","size_bytes":12000,"duration_ms":90000}"#,
        ))
        .expect("request");

    let response = router.oneshot(request).await.expect("response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let json: Value = serde_json::from_slice(&body).expect("json");
    assert!(json["id"]
        .as_str()
        .unwrap_or_default()
        .starts_with("debug-"));
    let DeliveryCommand::Candidate(candidate) = commands.recv().await.expect("candidate") else {
        panic!("expected candidate command");
    };
    assert_eq!(candidate.meta.urls, ["https://cdn.example/video.mp4"]);
    assert_eq!(candidate.meta.size_bytes, Some(12_000));
    assert_eq!(candidate.meta.duration_ms, Some(90_000));
    assert_eq!(candidate.meta.delivery, DeliveryKind::Progressive);
}

fn progressive_state(
    harness: &gateway_fixture::progressive::ProgressiveHarness,
) -> std::sync::Arc<ProgressiveState> {
    let (demand, _) = ghostr_delivery::playback_demand::demand_channel();
    std::sync::Arc::new(ProgressiveState {
        store: harness.store.clone(),
        demand,
        cache: harness.posts.clone(),
        network: harness.network.clone(),
        timing: Default::default(),
        debug_feed: ghostr_delivery::debug::feed::DebugFeed::new(
            ghostr_delivery::delivery_events::command_channel().0,
            Vec::new(),
        ),
    })
}
