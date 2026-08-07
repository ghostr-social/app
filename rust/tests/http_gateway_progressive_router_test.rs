mod support;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use rust_lib_ghostr::video::debug_network::NetworkThrottle;
use rust_lib_ghostr::video::hls_sessions::HlsSessions;
use rust_lib_ghostr::video::http_gateway::configured_router_with_progressive;
use rust_lib_ghostr::video::native_models::new_native_downloads;
use rust_lib_ghostr::video::partial_range_store::PartialRangeStore;
use rust_lib_ghostr::video::playback_demand::demand_channel;
use rust_lib_ghostr::video::progressive_posts::ServablePosts;
use rust_lib_ghostr::video::progressive_route::{ProgressiveState, ProgressiveTiming};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceExt;

fn standard_router() -> (Router, PathBuf) {
    let root = support::fixtures::temp_directory("progressive-router");
    let (demand, _) = demand_channel();
    #[cfg(feature = "video-debug-web")]
    let (delivery, _) = rust_lib_ghostr::video::delivery_events::command_channel();
    let progressive = Arc::new(ProgressiveState {
        store: Arc::new(PartialRangeStore::new(
            root.clone(),
            Arc::new(Mutex::new(0)),
        )),
        demand,
        cache: ServablePosts::new(),
        network: NetworkThrottle::new(),
        timing: ProgressiveTiming::default(),
        #[cfg(feature = "video-debug-web")]
        debug_feed: rust_lib_ghostr::video::debug_feed::DebugFeed::new(delivery, Vec::new()),
    });
    (
        configured_router_with_progressive(
            new_native_downloads(),
            HlsSessions::production(),
            support::fixtures::trusted_media_client(),
            progressive,
        ),
        root,
    )
}

#[tokio::test]
async fn ordinary_progressive_router_keeps_the_health_endpoint() {
    let (router, root) = standard_router();
    let request = Request::get("/status")
        .body(Body::empty())
        .expect("request");
    let response = router.oneshot(request).await.expect("response");

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    let _ = std::fs::remove_dir_all(root);
}
