#![cfg(feature = "video-debug-web")]

mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use gateway_fixture::commands::next_control;
use ghostr_delivery::delivery_events::{command_channel, DeliveryCommand};
use ghostr_discovery::cache::client_with_event_cache;
use ghostr_engine::DeliveryKind;
use ghostr_gateway::hls::sessions::HlsSessions;
use ghostr_gateway::progressive::route::ProgressiveState;
use ghostr_gateway::router::{configured_router_with_segmented_debug, GatewayRouterResources};
use serde_json::Value;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn debug_api_submits_a_registered_video_only_when_focused() {
    let harness = gateway_fixture::progressive::progressive_harness("debug-video-add");
    let (delivery, mut commands) = command_channel();
    let router = configured_router_with_segmented_debug(
        GatewayRouterResources::new(HlsSessions::production(), gateway_fixture::media_client()),
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

    let response = router.clone().oneshot(request).await.expect("response");

    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let json: Value = serde_json::from_slice(&body).expect("json");
    let id = json["id"].as_str().expect("id");
    assert!(id.starts_with("debug-"));
    let select = Request::put("/debug/api/focus")
        .header("content-type", "application/json")
        .body(Body::from(format!(r#"{{"id":"{id}"}}"#)))
        .expect("focus request");
    let selected = router.oneshot(select).await.expect("focus response");

    assert_eq!(selected.status(), StatusCode::NO_CONTENT);
    let DeliveryCommand::Focus(focus) = next_control(&mut commands).await else {
        panic!("expected focus command");
    };
    assert_eq!(focus.items[0].meta.urls, ["https://cdn.example/video.mp4"]);
    assert_eq!(focus.items[0].meta.size_bytes, Some(12_000));
    assert_eq!(focus.items[0].meta.duration_ms, Some(90_000));
    assert_eq!(focus.items[0].meta.delivery, DeliveryKind::Progressive);
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
        capabilities: harness.capabilities.clone(),
        debug_feed: ghostr_delivery::debug::feed::DebugFeed::new(
            ghostr_delivery::delivery_events::command_channel().0,
            Vec::new(),
        ),
    })
}
