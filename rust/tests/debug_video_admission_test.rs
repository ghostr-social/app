#![cfg(feature = "video-debug-web")]

mod support;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use rust_lib_ghostr::discovery::event_cache::client_with_event_cache;
use rust_lib_ghostr::engine::DeliveryKind;
use rust_lib_ghostr::video::delivery_events::{command_channel, DeliveryCommand};
use rust_lib_ghostr::video::hls_sessions::HlsSessions;
use rust_lib_ghostr::video::http_gateway::configured_router_with_progressive_debug;
use rust_lib_ghostr::video::native_models::new_native_downloads;
use rust_lib_ghostr::video::progressive_route::ProgressiveState;
use serde_json::Value;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn debug_api_registers_a_video_with_the_rust_delivery_engine() {
    let harness = support::progressive::progressive_harness("debug-video-add");
    let (delivery, mut commands) = command_channel();
    let router = configured_router_with_progressive_debug(
        new_native_downloads(),
        HlsSessions::production(),
        support::fixtures::trusted_media_client(),
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
    harness: &support::progressive::ProgressiveHarness,
) -> std::sync::Arc<ProgressiveState> {
    let (demand, _) = rust_lib_ghostr::video::playback_demand::demand_channel();
    std::sync::Arc::new(ProgressiveState {
        store: harness.store.clone(),
        demand,
        cache: harness.posts.clone(),
        network: harness.network.clone(),
        timing: Default::default(),
        debug_feed: rust_lib_ghostr::video::debug_feed::DebugFeed::new(
            rust_lib_ghostr::video::delivery_events::command_channel().0,
            Vec::new(),
        ),
    })
}
