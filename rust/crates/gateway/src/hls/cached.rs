use crate::hls::asset_request::{AssetRangeRequest, ResolvedAssetRange};
use axum::body::Body;
use axum::http::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE};
use axum::http::{Response, StatusCode};
use bytes::Bytes;
use ghostr_delivery::segmented::CachedHlsObject;
use std::sync::Arc;

#[cfg(test)]
mod tests;
#[cfg(test)]
#[path = "cached/zero_copy_test.rs"]
mod zero_copy_test;

pub(super) fn response(
    object: CachedHlsObject,
    request: AssetRangeRequest,
) -> Result<Response<Body>, StatusCode> {
    let total = object.body.len() as u64;
    match request.resolve(total) {
        ResolvedAssetRange::Full => build(&object, 0, total, false),
        ResolvedAssetRange::Partial { start, end } => build(&object, start, end, true),
        ResolvedAssetRange::Unsatisfiable => unsatisfiable(total),
    }
}

fn build(
    object: &CachedHlsObject,
    start: u64,
    end: u64,
    partial: bool,
) -> Result<Response<Body>, StatusCode> {
    let range = start as usize..end as usize;
    let length = object
        .body
        .get(range.clone())
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .len();
    let body = Bytes::from_owner(Arc::clone(&object.body)).slice(range);
    let status = if partial {
        StatusCode::PARTIAL_CONTENT
    } else {
        StatusCode::OK
    };
    let mut response = Response::builder()
        .status(status)
        .header(ACCEPT_RANGES, "bytes")
        .header(CONTENT_LENGTH, length);
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
        .body(Body::from(body))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn unsatisfiable(total: u64) -> Result<Response<Body>, StatusCode> {
    Response::builder()
        .status(StatusCode::RANGE_NOT_SATISFIABLE)
        .header(CONTENT_LENGTH, 0)
        .header(CONTENT_RANGE, format!("bytes */{total}"))
        .body(Body::empty())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
