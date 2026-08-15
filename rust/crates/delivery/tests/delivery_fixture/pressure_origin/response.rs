use super::OriginState;
use axum::body::Body;
use axum::extract::State;
use axum::http::{header, HeaderMap, Method, StatusCode};
use axum::response::Response;
use std::sync::atomic::Ordering;

const MEDIA: &[u8] = b"0123456789abcdef";

pub(super) async fn response(
    State(state): State<OriginState>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    if method == Method::HEAD {
        return reply(StatusCode::OK, 0, MEDIA.len() - 1, Body::empty());
    }
    let (start, end) = requested(&headers);
    state.requests.lock().expect("requests").push((start, end));
    if state.bodies.fetch_add(1, Ordering::SeqCst) == 0 {
        let signal = state.started.lock().expect("body signal").take();
        if let Some(signal) = signal {
            signal.send((start, end)).ok();
        }
        state.release.notified().await;
    }
    reply(
        StatusCode::PARTIAL_CONTENT,
        start,
        end,
        Body::from(MEDIA[start..=end].to_vec()),
    )
}

fn requested(headers: &HeaderMap) -> (usize, usize) {
    let range = headers[header::RANGE].to_str().expect("range header");
    let (start, end) = range.trim_start_matches("bytes=").split_once('-').unwrap();
    let start = start.parse().unwrap();
    let end = end.parse().unwrap_or(MEDIA.len() - 1).min(MEDIA.len() - 1);
    (start, end)
}

fn reply(status: StatusCode, start: usize, end: usize, body: Body) -> Response {
    let mut builder = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::ETAG, "\"fixture-pressure\"")
        .header(header::CONTENT_LENGTH, end - start + 1);
    if status == StatusCode::PARTIAL_CONTENT {
        builder = builder.header(header::CONTENT_RANGE, format!("bytes {start}-{end}/16"));
    }
    builder.body(body).expect("pressure response")
}
