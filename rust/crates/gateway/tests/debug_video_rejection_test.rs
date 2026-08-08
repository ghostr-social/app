#![cfg(feature = "video-debug-web")]

mod gateway_fixture;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use ghostr_delivery::delivery_events::command_channel;
use ghostr_discovery::cache::client_with_event_cache;
use ghostr_gateway::hls::sessions::HlsSessions;
use ghostr_gateway::router::configured_router_with_progressive_debug;
use tower::ServiceExt;

#[tokio::test]
async fn debug_api_rejects_a_non_http_video_url() {
    let harness = gateway_fixture::progressive::progressive_harness("debug-video-reject");
    let (delivery, _) = command_channel();
    let (demand, _) = ghostr_delivery::playback_demand::demand_channel();
    let state = std::sync::Arc::new(ghostr_gateway::progressive::route::ProgressiveState {
        store: harness.store,
        demand,
        cache: harness.posts,
        network: harness.network,
        timing: Default::default(),
        debug_feed: ghostr_delivery::debug::feed::DebugFeed::new(delivery.clone(), Vec::new()),
    });
    let router = configured_router_with_progressive_debug(
        HlsSessions::production(),
        gateway_fixture::media_client(),
        state,
        delivery,
        std::sync::Arc::new(client_with_event_cache()),
    );
    let request = Request::builder()
        .method("POST")
        .uri("/debug/api/videos")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"url":"file:///tmp/video.mp4"}"#))
        .expect("request");

    let response = router.oneshot(request).await.expect("response");

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}
