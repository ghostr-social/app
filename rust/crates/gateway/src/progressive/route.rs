use crate::progressive::capabilities::ProgressiveCapabilities;
use crate::progressive::range_header::{self, ResolvedRange};
use crate::progressive::stream::body_for_span;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::header::{
    ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE, RETRY_AFTER,
};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use ghostr_delivery::cache_registry::CacheRegistry;
#[cfg(all(
    feature = "video-debug-web",
    debug_assertions,
    not(any(target_os = "android", target_os = "ios"))
))]
use ghostr_delivery::debug::feed::DebugFeed;
use ghostr_delivery::debug::network::NetworkThrottle;
use ghostr_delivery::playback_demand::DemandSender;
use ghostr_partial_store::partial_range_store::PartialRangeStore;
use serde::Deserialize;
use std::ops::Range;
use std::sync::Arc;
use std::time::Duration;

mod authority;
mod snapshot;
use authority::{refresh_current_asset, require_current_asset, require_servable};
use snapshot::{awaited_media_snapshot, VideoSnapshot};

const VIDEO_MIME: &str = "video/mp4";

/// How long progressive serving waits on the store before giving up.
#[derive(Clone, Copy, Debug)]
pub struct ProgressiveTiming {
    /// Wait for the total length to be learned before answering 503.
    unknown_length_wait: Duration,
    /// Abort a stalled stream once no byte lands for this long.
    pub(crate) idle_timeout: Duration,
}

impl Default for ProgressiveTiming {
    fn default() -> Self {
        Self {
            unknown_length_wait: Duration::from_secs(2),
            idle_timeout: Duration::from_secs(15),
        }
    }
}

/// Dependencies of the progressive `/video.mp4` route.
pub struct ProgressiveState {
    pub store: Arc<PartialRangeStore>,
    pub demand: DemandSender,
    pub cache: CacheRegistry,
    pub network: NetworkThrottle,
    pub timing: ProgressiveTiming,
    pub capabilities: ProgressiveCapabilities,
    #[cfg(all(
        feature = "video-debug-web",
        debug_assertions,
        not(any(target_os = "android", target_os = "ios"))
    ))]
    pub debug_feed: DebugFeed,
}

#[derive(Deserialize)]
struct VideoQuery {
    id: String,
    cap: Option<String>,
}

pub(crate) fn router(state: Arc<ProgressiveState>) -> Router {
    Router::new()
        .route("/video.mp4", get(serve))
        .with_state(state)
}

async fn serve(
    State(state): State<Arc<ProgressiveState>>,
    Query(query): Query<VideoQuery>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    require_servable(&state, &query).await?;
    if !refresh_current_asset(&state, &query).await? {
        return retry_later();
    }
    let Some(stored) = awaited_media_snapshot(&state, &query.id).await? else {
        return retry_later();
    };
    require_current_asset(&state, &query, &stored).await?;
    let snapshot = VideoSnapshot::from_stored(query.id, stored);
    let mut response = match range_header::resolve(headers.get(RANGE), snapshot.total) {
        ResolvedRange::Full => full_response(state, snapshot),
        ResolvedRange::Partial { start, end } => partial_response(state, snapshot, start..end),
        ResolvedRange::Unsatisfiable => unsatisfiable_response(snapshot.total),
    }?;
    response
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static(VIDEO_MIME));
    Ok(response)
}

fn retry_later() -> Result<Response, StatusCode> {
    Response::builder()
        .status(StatusCode::SERVICE_UNAVAILABLE)
        .header(RETRY_AFTER, "1")
        .body(Body::empty())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn full_response(
    state: Arc<ProgressiveState>,
    snapshot: VideoSnapshot,
) -> Result<Response, StatusCode> {
    let body = body_for_span(state, snapshot.source, 0..snapshot.total);
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, VIDEO_MIME)
        .header(CONTENT_LENGTH, snapshot.total)
        .header(ACCEPT_RANGES, "bytes")
        .body(body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn partial_response(
    state: Arc<ProgressiveState>,
    snapshot: VideoSnapshot,
    span: Range<u64>,
) -> Result<Response, StatusCode> {
    let content_range = format!("bytes {}-{}/{}", span.start, span.end - 1, snapshot.total);
    let length = span.end - span.start;
    let body = body_for_span(state, snapshot.source, span);
    Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(CONTENT_TYPE, VIDEO_MIME)
        .header(CONTENT_LENGTH, length)
        .header(CONTENT_RANGE, content_range)
        .header(ACCEPT_RANGES, "bytes")
        .body(body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn unsatisfiable_response(total: u64) -> Result<Response, StatusCode> {
    Response::builder()
        .status(StatusCode::RANGE_NOT_SATISFIABLE)
        .header(CONTENT_RANGE, format!("bytes */{total}"))
        .body(Body::empty())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
