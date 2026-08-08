mod gateway_fixture;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_delivery::playback_demand::demand_channel;
use ghostr_delivery::progressive_posts::ServablePosts;
use ghostr_gateway::hls::sessions::HlsSessions;
use ghostr_gateway::progressive::route::{ProgressiveState, ProgressiveTiming};
use ghostr_gateway::router::configured_router_with_progressive;
use ghostr_partial_store::partial_range_store::capacity::StoreCapacity;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceExt;

fn standard_router() -> (Router, PathBuf) {
    let root = gateway_fixture::temp_directory("progressive-router");
    let (demand, _) = demand_channel();
    #[cfg(feature = "video-debug-web")]
    let (delivery, _) = ghostr_delivery::delivery_events::command_channel();
    let progressive = Arc::new(ProgressiveState {
        store: Arc::new(PartialRangeStore::with_capacity(
            root.clone(),
            Arc::new(Mutex::new(0)),
            StoreCapacity::system(u64::MAX),
        )),
        demand,
        cache: ServablePosts::new(),
        network: NetworkThrottle::new(),
        timing: ProgressiveTiming::default(),
        #[cfg(feature = "video-debug-web")]
        debug_feed: ghostr_delivery::debug::feed::DebugFeed::new(delivery, Vec::new()),
    });
    (
        configured_router_with_progressive(
            HlsSessions::production(),
            gateway_fixture::media_client(),
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
