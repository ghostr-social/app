#![cfg(feature = "video-debug-web")]

mod gateway_fixture;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use axum::Router;
use gateway_fixture::delivery::{start_delivery, DeliveryFixture};
use gateway_fixture::media_origin::MediaOrigin;
use ghostr_delivery::debug::feed::DebugFeed;
use ghostr_discovery::cache::client_with_event_cache;
use ghostr_gateway::hls::sessions::HlsSessions;
use ghostr_gateway::progressive::capabilities::ProgressiveCapabilities;
use ghostr_gateway::progressive::route::ProgressiveState;
use ghostr_gateway::router::configured_router_with_progressive_debug;
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn registrations_wait_for_focus_before_starting_origin_work() {
    let origin = MediaOrigin::serve().await;
    let delivery = start_delivery("debug-passive-admission");
    let router = debug_router(&delivery);
    let mut ids = Vec::new();
    for index in 0..4 {
        ids.push(register(&router, origin.url(&format!("video-{index}"))).await);
    }

    origin.assert_no_get().await;
    select(&router, &ids[0]).await;
    origin.wait_for_gets(&["video-0"]).await;

    delivery.handle.clear().await.unwrap();
    std::fs::remove_dir_all(&delivery.root).ok();
}

fn debug_router(delivery: &DeliveryFixture) -> Router {
    let progressive = Arc::new(ProgressiveState {
        store: delivery.store.clone(),
        demand: delivery.demand.clone(),
        cache: delivery.cache.clone(),
        network: delivery.network.clone(),
        timing: Default::default(),
        capabilities: ProgressiveCapabilities::production(),
        debug_feed: DebugFeed::new(delivery.handle.clone(), Vec::new()),
    });
    configured_router_with_progressive_debug(
        HlsSessions::production(),
        gateway_fixture::media_client(),
        progressive,
        delivery.handle.clone(),
        Arc::new(client_with_event_cache()),
    )
}

async fn register(router: &Router, url: String) -> String {
    let request = Request::post("/debug/api/videos")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({"url": url, "size_bytes": 64, "duration_ms": 1_000}).to_string(),
        ))
        .unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: Value = serde_json::from_slice(&body).unwrap();
    json["id"].as_str().unwrap().to_owned()
}

async fn select(router: &Router, id: &str) {
    let request = Request::put("/debug/api/focus")
        .header("content-type", "application/json")
        .body(Body::from(json!({"id": id}).to_string()))
        .unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}
