#![cfg(feature = "video-debug-web")]

mod support;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use rust_lib_ghostr::discovery::event_cache::client_with_event_cache;
use rust_lib_ghostr::video::delivery_events::command_channel;
use rust_lib_ghostr::video::hls_sessions::HlsSessions;
use rust_lib_ghostr::video::http_gateway::configured_router_with_progressive_debug;
use rust_lib_ghostr::video::native_models::new_native_downloads;
use tower::ServiceExt;

#[tokio::test]
async fn debug_api_rejects_a_non_http_video_url() {
    let harness = support::progressive::progressive_harness("debug-video-reject");
    let (delivery, _) = command_channel();
    let (demand, _) = rust_lib_ghostr::video::playback_demand::demand_channel();
    let state = std::sync::Arc::new(
        rust_lib_ghostr::video::progressive_route::ProgressiveState {
            store: harness.store,
            demand,
            cache: harness.posts,
            network: harness.network,
            timing: Default::default(),
            debug_feed: rust_lib_ghostr::video::debug_feed::DebugFeed::new(
                delivery.clone(),
                Vec::new(),
            ),
        },
    );
    let router = configured_router_with_progressive_debug(
        new_native_downloads(),
        HlsSessions::production(),
        support::fixtures::trusted_media_client(),
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
