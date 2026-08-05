use axum::body::Body;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::Response;

pub(super) fn full(bytes: &[u8]) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::CONTENT_LENGTH, bytes.len())
        .header(header::ACCEPT_RANGES, "bytes")
        .body(Body::from(bytes.to_vec()))
        .expect("full response")
}

pub(super) fn partial(bytes: &[u8], start: u64, end: u64) -> Response {
    let slice = bytes
        .get(start as usize..(end + 1) as usize)
        .unwrap_or(&[])
        .to_vec();
    Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::CONTENT_LENGTH, slice.len())
        .header(
            header::CONTENT_RANGE,
            format!("bytes {start}-{end}/{}", bytes.len()),
        )
        .body(Body::from(slice))
        .expect("partial response")
}

pub(super) fn requested(headers: &HeaderMap, len: u64) -> Option<(u64, u64)> {
    let value = headers.get(header::RANGE)?.to_str().ok()?;
    let (start, end) = value.strip_prefix("bytes=")?.split_once('-')?;
    let last = len.saturating_sub(1);
    let start = start.parse().ok()?;
    let end: u64 = end.parse().unwrap_or(last);
    Some((start, end.min(last)))
}
