use super::OriginState;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, Method, StatusCode};
use axum::response::Response;
use core::sync::atomic::Ordering;

const MEDIA_BYTES: u64 = 64;

pub(super) async fn media(
    Path(kind): Path<String>,
    State(state): State<OriginState>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    if kind == "cooling" {
        state.failures.fetch_add(1, Ordering::SeqCst);
        return empty_response(StatusCode::INTERNAL_SERVER_ERROR, 0);
    }
    if method == Method::HEAD {
        return empty_response(StatusCode::OK, MEDIA_BYTES);
    }
    state.useful.fetch_add(1, Ordering::SeqCst);
    state.started.add_permits(1);
    wait_for_release(&state).await;
    media_response(&headers)
}

async fn wait_for_release(state: &OriginState) {
    state
        .release
        .acquire()
        .await
        .expect("valid test fixture")
        .forget();
}

fn empty_response(status: StatusCode, length: u64) -> Response {
    response_builder(status, length)
        .body(Body::empty())
        .expect("valid test fixture")
}

fn media_response(headers: &HeaderMap) -> Response {
    let Some(range) = headers.get(header::RANGE) else {
        return response_builder(StatusCode::OK, MEDIA_BYTES)
            .body(Body::from(vec![7; MEDIA_BYTES as usize]))
            .expect("valid test fixture");
    };
    let range = range.to_str().expect("range header");
    let (start, end) = requested_range(range);
    response_builder(StatusCode::PARTIAL_CONTENT, end - start + 1)
        .header(header::CONTENT_RANGE, format!("bytes {start}-{end}/64"))
        .body(Body::from(vec![7; (end - start + 1) as usize]))
        .expect("valid test fixture")
}

fn requested_range(value: &str) -> (u64, u64) {
    let (start, end) = value
        .trim_start_matches("bytes=")
        .split_once('-')
        .expect("valid test fixture");
    let start = start.parse().expect("valid test fixture");
    let end = end.parse::<u64>().unwrap_or(63).min(63);
    (start, end)
}

fn response_builder(status: StatusCode, length: u64) -> axum::http::response::Builder {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::ETAG, "\"fixture-cooling\"")
        .header(header::CONTENT_LENGTH, length)
}
