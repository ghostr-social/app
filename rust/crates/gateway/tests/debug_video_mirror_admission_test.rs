#![cfg(feature = "video-debug-web")]

mod gateway_fixture;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use ghostr_delivery::delivery_events::{command_channel, DeliveryCommand};
use ghostr_discovery::cache::client_with_event_cache;
use ghostr_gateway::hls::sessions::HlsSessions;
use ghostr_gateway::progressive::route::ProgressiveState;
use ghostr_gateway::router::configured_router_with_progressive_debug;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn debug_video_registration_preserves_ordered_mirrors() {
    let harness = gateway_fixture::progressive::progressive_harness("debug-video-mirror");
    let (delivery, mut commands) = command_channel();
    let router = configured_router_with_progressive_debug(
        HlsSessions::production(),
        gateway_fixture::media_client(),
        progressive_state(&harness),
        delivery,
        Arc::new(client_with_event_cache()),
    );
    let body = r#"{"url":"https://primary.example/video.mp4","mirrors":["https://mirror.example/video.mp4"]}"#;

    let response = router
        .clone()
        .oneshot(
            Request::post("/debug/api/videos")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let focus = format!(r#"{{"id":"{}"}}"#, json["id"].as_str().unwrap());
    let selected = router
        .oneshot(
            Request::put("/debug/api/focus")
                .header("content-type", "application/json")
                .body(Body::from(focus))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(selected.status(), StatusCode::NO_CONTENT);
    let DeliveryCommand::Focus(focus) = commands.receivers().0.recv().await.unwrap() else {
        panic!("expected focus");
    };
    assert_eq!(
        focus.items[0].meta.urls,
        [
            "https://primary.example/video.mp4",
            "https://mirror.example/video.mp4",
        ]
    );
}

fn progressive_state(
    harness: &gateway_fixture::progressive::ProgressiveHarness,
) -> Arc<ProgressiveState> {
    let (demand, _) = ghostr_delivery::playback_demand::demand_channel();
    Arc::new(ProgressiveState {
        store: harness.store.clone(),
        demand,
        cache: harness.posts.clone(),
        network: harness.network.clone(),
        timing: Default::default(),
        capabilities: harness.capabilities.clone(),
        debug_feed: harness.debug_feed.clone(),
    })
}
