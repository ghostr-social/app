use crate::video::partial_range_store::PartialRangeStore;
use crate::video::playback_demand::DemandSender;
use crate::video::progressive_posts::ServablePosts;
use crate::video::progressive_stream::body_for_span;
use crate::video::range_header::{self, ResolvedRange};
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::header::{
    ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE, RETRY_AFTER,
};
use axum::http::{HeaderMap, StatusCode};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use serde::Deserialize;
use std::ops::Range;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{timeout_at, Instant};

const VIDEO_MIME: &str = "video/mp4";

/// How long progressive serving waits on the store before giving up.
#[derive(Clone, Copy, Debug)]
pub struct ProgressiveTiming {
    /// Wait for the total length to be learned before answering 503.
    pub unknown_length_wait: Duration,
    /// Abort a stalled stream once no byte lands for this long.
    pub idle_timeout: Duration,
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
    pub posts: ServablePosts,
    pub timing: ProgressiveTiming,
}

#[derive(Deserialize)]
struct VideoQuery {
    id: String,
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
    if !state.posts.contains(&query.id) {
        return Err(StatusCode::NOT_FOUND);
    }
    let Some(total) = awaited_total_len(&state, &query.id).await? else {
        return retry_later();
    };
    match range_header::resolve(headers.get(RANGE), total) {
        ResolvedRange::Full => full_response(state, query.id, total),
        ResolvedRange::Partial { start, end } => {
            partial_response(state, query.id, total, start..end)
        }
        ResolvedRange::Unsatisfiable => unsatisfiable_response(total),
    }
}

/// The total length must be known (probe or imeta) before the first serve;
/// wait briefly for the store to learn it, then hand back `None` for a 503.
async fn awaited_total_len(
    state: &Arc<ProgressiveState>,
    id: &str,
) -> Result<Option<u64>, StatusCode> {
    let deadline = Instant::now() + state.timing.unknown_length_wait;
    let notify = state.store.change_notifier();
    loop {
        let changed = notify.notified();
        let known = state
            .store
            .total_len(id)
            .await
            .map_err(|_| StatusCode::NOT_FOUND)?;
        if known.is_some() {
            return Ok(known);
        }
        if timeout_at(deadline, changed).await.is_err() {
            return Ok(None);
        }
    }
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
    id: String,
    total: u64,
) -> Result<Response, StatusCode> {
    let body = body_for_span(state, id, 0..total);
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, VIDEO_MIME)
        .header(CONTENT_LENGTH, total)
        .header(ACCEPT_RANGES, "bytes")
        .body(body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn partial_response(
    state: Arc<ProgressiveState>,
    id: String,
    total: u64,
    span: Range<u64>,
) -> Result<Response, StatusCode> {
    let content_range = format!("bytes {}-{}/{}", span.start, span.end - 1, total);
    let length = span.end - span.start;
    let body = body_for_span(state, id, span);
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
