use crate::progressive::range_header::{resolve, ResolvedRange};
use axum::body::Body;
use axum::http::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE};
use axum::http::{HeaderMap, Response, StatusCode};
use ghostr_delivery::segmented::CachedHlsObject;

pub(super) fn response(
    object: CachedHlsObject,
    headers: &HeaderMap,
) -> Result<Response<Body>, StatusCode> {
    let total = object.body.len() as u64;
    match resolve(headers.get(RANGE), total) {
        ResolvedRange::Full => build(StatusCode::OK, &object, 0, total, false),
        ResolvedRange::Partial { start, end } => {
            build(StatusCode::PARTIAL_CONTENT, &object, start, end, true)
        }
        ResolvedRange::Unsatisfiable => unsatisfiable(total),
    }
}

fn build(
    status: StatusCode,
    object: &CachedHlsObject,
    start: u64,
    end: u64,
    partial: bool,
) -> Result<Response<Body>, StatusCode> {
    let span = object
        .body
        .get(start as usize..end as usize)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut response = Response::builder()
        .status(status)
        .header(ACCEPT_RANGES, "bytes")
        .header(CONTENT_LENGTH, span.len());
    if let Some(content_type) = &object.content_type {
        response = response.header(CONTENT_TYPE, content_type);
    }
    if partial {
        response = response.header(
            CONTENT_RANGE,
            format!("bytes {start}-{}/{}", end - 1, object.body.len()),
        );
    }
    response
        .body(Body::from(span.to_vec()))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn unsatisfiable(total: u64) -> Result<Response<Body>, StatusCode> {
    Response::builder()
        .status(StatusCode::RANGE_NOT_SATISFIABLE)
        .header(CONTENT_RANGE, format!("bytes */{total}"))
        .body(Body::empty())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
