use axum::body::Body;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::Response;

const MEDIA_LEN: u64 = 64;

pub(super) fn metadata() -> Response {
    Response::builder()
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::CONTENT_LENGTH, MEDIA_LEN)
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::ETAG, "\"fixture-gateway-media\"")
        .body(Body::empty())
        .unwrap()
}

pub(super) fn partial(headers: &HeaderMap) -> Response {
    let (start, end) = requested_span(headers);
    Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(
            header::CONTENT_RANGE,
            format!("bytes {start}-{end}/{MEDIA_LEN}"),
        )
        .header(header::CONTENT_LENGTH, end - start + 1)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::ETAG, "\"fixture-gateway-media\"")
        .body(Body::from(vec![1; (end - start + 1) as usize]))
        .unwrap()
}

fn requested_span(headers: &HeaderMap) -> (u64, u64) {
    let Some(value) = headers
        .get(header::RANGE)
        .and_then(|value| value.to_str().ok())
    else {
        return (0, MEDIA_LEN - 1);
    };
    let Some((start, end)) = value.trim_start_matches("bytes=").split_once('-') else {
        return (0, MEDIA_LEN - 1);
    };
    let start = start.parse().unwrap_or(0).min(MEDIA_LEN - 1);
    let end = end.parse().unwrap_or(MEDIA_LEN - 1).min(MEDIA_LEN - 1);
    (start, end.max(start))
}
