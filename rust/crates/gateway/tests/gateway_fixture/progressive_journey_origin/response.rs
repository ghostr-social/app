use axum::body::Body;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::Response;

pub(super) fn rejected_head() -> Response {
    Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .body(Body::empty())
        .expect("rejected HEAD")
}

pub(super) fn lengthless_head() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::ETAG, "\"fixture-progressive-journey\"")
        .body(Body::empty())
        .expect("lengthless HEAD")
}

pub(super) fn failed_body() -> Response {
    Response::builder()
        .status(StatusCode::SERVICE_UNAVAILABLE)
        .body(Body::empty())
        .expect("failed body")
}

pub(super) fn range_opaque_head(total: usize) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::CONTENT_LENGTH, total)
        .header(header::ETAG, "\"fixture-progressive-journey\"")
        .body(Body::empty())
        .expect("range-opaque HEAD")
}

pub(super) fn partial(bytes: &[u8], headers: &HeaderMap) -> Response {
    let (start, end) = requested_span(headers, bytes.len() as u64);
    Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::CONTENT_LENGTH, end - start)
        .header(header::ETAG, "\"fixture-progressive-journey\"")
        .header(
            header::CONTENT_RANGE,
            format!("bytes {}-{}/{}", start, end - 1, bytes.len()),
        )
        .body(Body::from(bytes[start as usize..end as usize].to_vec()))
        .expect("partial response")
}

fn requested_span(headers: &HeaderMap, total: u64) -> (u64, u64) {
    let value = headers
        .get(header::RANGE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("bytes=0-");
    let (start, end) = value
        .trim_start_matches("bytes=")
        .split_once('-')
        .unwrap_or(("0", ""));
    let start = start.parse::<u64>().unwrap_or(0).min(total - 1);
    let end = end
        .parse::<u64>()
        .map(|value| value + 1)
        .unwrap_or(total)
        .min(total);
    (start, end.max(start + 1))
}
