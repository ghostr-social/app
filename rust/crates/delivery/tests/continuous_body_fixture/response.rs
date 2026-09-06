use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderMap, Method};
use axum::response::Response;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

const MODIFIED: &str = "Thu, 03 Sep 2026 23:47:55 GMT";

pub(super) async fn dated_answer(
    state: State<Arc<AtomicUsize>>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    let mut response = answer(state, method, headers).await;
    response
        .headers_mut()
        .insert("last-modified", MODIFIED.parse().expect("date header"));
    response
}

pub(super) async fn answer(
    State(whole): State<Arc<AtomicUsize>>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    let response = Response::builder()
        .header("content-type", "video/mp4")
        .header("etag", "W/\"realistic-weak-validator\"")
        .header("cache-control", "public, max-age=3600");
    if let Some(range) = headers.get("range") {
        return partial(response, range.to_str().expect("range"));
    }
    let response = response.header("content-length", super::TOTAL);
    if method == Method::HEAD {
        return response.body(Body::empty()).expect("head");
    }
    whole.fetch_add(1, Ordering::Relaxed);
    response
        .body(super::body::held_response())
        .expect("whole response")
}

fn partial(response: axum::http::response::Builder, range: &str) -> Response {
    let (start, end) = range
        .trim_start_matches("bytes=")
        .split_once('-')
        .expect("bounds");
    let start: u64 = start.parse().expect("start");
    let end: u64 = end.parse().expect("end");
    let length = end.min(super::TOTAL - 1) - start + 1;
    response
        .status(206)
        .header(
            "content-range",
            format!("bytes {start}-{}/{}", start + length - 1, super::TOTAL),
        )
        .header("content-length", length)
        .body(Body::from(vec![7; length as usize]))
        .expect("partial response")
}
