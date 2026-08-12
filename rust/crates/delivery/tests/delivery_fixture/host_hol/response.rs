use axum::body::Body;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::Response;

pub(super) fn head_response() -> Response {
    Response::builder()
        .header(header::CONTENT_LENGTH, 64)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::ACCEPT_RANGES, "bytes")
        .body(Body::empty())
        .unwrap()
}

pub(super) fn range_response(headers: &HeaderMap) -> Response {
    let (start, end) = requested(headers);
    Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::ACCEPT_RANGES, "bytes")
        .header(
            header::CONTENT_RANGE,
            format!("bytes {start}-{}/64", end - 1),
        )
        .body(Body::from(vec![1; (end - start) as usize]))
        .unwrap()
}

fn requested(headers: &HeaderMap) -> (u64, u64) {
    let value = headers
        .get(header::RANGE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("bytes=0-7");
    let (start, end) = value.trim_start_matches("bytes=").split_once('-').unwrap();
    let start = start.parse().unwrap();
    let end = end.parse::<u64>().unwrap_or(63).min(63) + 1;
    (start, end)
}
