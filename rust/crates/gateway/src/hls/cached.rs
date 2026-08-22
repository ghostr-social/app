use crate::hls::asset_request::{AssetRangeRequest, ResolvedAssetRange};
use axum::body::Body;
use axum::http::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE};
use axum::http::{Response, StatusCode};
use ghostr_delivery::segmented::CachedHlsObject;

#[cfg(test)]
mod tests;

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
    let span = object
        .body
        .get(start as usize..end as usize)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let status = if partial {
        StatusCode::PARTIAL_CONTENT
    } else {
        StatusCode::OK
    };
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
        .header(CONTENT_LENGTH, 0)
        .header(CONTENT_RANGE, format!("bytes */{total}"))
        .body(Body::empty())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
