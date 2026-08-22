use axum::body::Body;
use axum::http::{header, Response, StatusCode};

pub(super) fn head(total: u64) -> Response<Body> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CONTENT_LENGTH, total)
        .header(header::ETAG, "\"hedge-tail-fixture\"")
        .body(Body::empty())
        .unwrap()
}

pub(super) fn partial(total: u64, bytes: u64) -> Response<Body> {
    Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CONTENT_LENGTH, bytes)
        .header(
            header::CONTENT_RANGE,
            format!("bytes 0-{}/{}", bytes - 1, total),
        )
        .header(header::ETAG, "\"hedge-tail-fixture\"")
        .body(Body::from(vec![7; bytes as usize]))
        .unwrap()
}
