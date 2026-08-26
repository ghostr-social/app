use super::INIT_BYTES;
use axum::body::Body;
use axum::http::{header, Response};

pub(super) const MANIFEST: &[u8] =
    b"#EXTM3U\n#EXT-X-MAP:URI=\"init.mp4\"\n#EXTINF:4,\nsegment.m4s\n#EXT-X-ENDLIST\n";

pub(super) fn partial(
    start: usize,
    requested_end: usize,
    generation: u8,
    total: usize,
) -> Response<Body> {
    let end = requested_end.min(total - 1);
    let value = generation.saturating_add(6);
    Response::builder()
        .status(206)
        .header(
            header::CONTENT_RANGE,
            format!("bytes {start}-{end}/{total}"),
        )
        .header(header::CONTENT_LENGTH, end - start + 1)
        .header(header::ETAG, format!("\"init-v{generation}\""))
        .body(Body::from(vec![value; end - start + 1]))
        .expect("valid test fixture")
}

pub(super) fn changed_full(generation: u8) -> Response<Body> {
    let value = generation.saturating_add(6);
    Response::builder()
        .header(header::CONTENT_LENGTH, INIT_BYTES)
        .header(header::ETAG, format!("\"init-v{generation}\""))
        .body(Body::from(vec![value; INIT_BYTES]))
        .expect("valid test fixture")
}

pub(super) fn partial_with_etag(
    start: usize,
    requested_end: usize,
    total: usize,
    etag: &'static str,
) -> Response<Body> {
    let end = requested_end.min(total - 1);
    Response::builder()
        .status(206)
        .header(
            header::CONTENT_RANGE,
            format!("bytes {start}-{end}/{total}"),
        )
        .header(header::CONTENT_LENGTH, end - start + 1)
        .header(header::ETAG, format!("\"{etag}\""))
        .body(Body::from(vec![8; end - start + 1]))
        .expect("valid test fixture")
}

pub(super) fn unsatisfied(total: usize, generation: u8) -> Response<Body> {
    Response::builder()
        .status(416)
        .header(header::CONTENT_RANGE, format!("bytes */{total}"))
        .header(header::CONTENT_LENGTH, 0)
        .header(header::ETAG, format!("\"init-v{generation}\""))
        .body(Body::empty())
        .expect("valid test fixture")
}

pub(super) fn full(body: &'static [u8], etag: &'static str) -> Response<Body> {
    Response::builder()
        .header(header::CONTENT_LENGTH, body.len())
        .header(header::ETAG, format!("\"{etag}\""))
        .body(Body::from(body))
        .expect("valid test fixture")
}
