use axum::body::Body;
use axum::http::header::RANGE;
use axum::http::Request;
use axum::Router;
use rust_lib_ghostr::video::hls_sessions::HlsSessions;
use rust_lib_ghostr::video::http_gateway::configured_router_with_progressive;
use rust_lib_ghostr::video::native_models::new_native_downloads;
use rust_lib_ghostr::video::partial_range_store::PartialRangeStore;
use rust_lib_ghostr::video::playback_demand::{demand_channel, DemandReceiver};
use rust_lib_ghostr::video::progressive_posts::ServablePosts;
use rust_lib_ghostr::video::progressive_route::{ProgressiveState, ProgressiveTiming};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ProgressiveHarness {
    pub router: Router,
    pub store: Arc<PartialRangeStore>,
    pub posts: ServablePosts,
    pub demand: DemandReceiver,
    pub root: PathBuf,
}

pub fn progressive_harness(prefix: &str) -> ProgressiveHarness {
    progressive_harness_with_timing(prefix, ProgressiveTiming::default())
}

pub fn progressive_harness_with_timing(
    prefix: &str,
    timing: ProgressiveTiming,
) -> ProgressiveHarness {
    let root = super::fixtures::temp_directory(prefix);
    let store = Arc::new(PartialRangeStore::new(
        root.clone(),
        Arc::new(Mutex::new(0)),
    ));
    let posts = ServablePosts::new();
    let (sender, demand) = demand_channel();
    let state = Arc::new(ProgressiveState {
        store: store.clone(),
        demand: sender,
        posts: posts.clone(),
        timing,
    });
    let router = configured_router_with_progressive(
        new_native_downloads(),
        HlsSessions::production(),
        super::fixtures::trusted_media_client(),
        state,
    );
    ProgressiveHarness {
        router,
        store,
        posts,
        demand,
        root,
    }
}

pub fn video_request(id: &str, range: Option<&str>) -> Request<Body> {
    let builder = Request::builder().uri(format!("/video.mp4?id={id}"));
    let builder = match range {
        Some(value) => builder.header(RANGE, value),
        None => builder,
    };
    builder.body(Body::empty()).expect("request")
}
