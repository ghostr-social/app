use super::OriginState;
use axum::body::Body;
use axum::extract::State;
use axum::http::header::{CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, ETAG, RANGE};
use axum::http::{HeaderMap, Response, StatusCode};
use core::sync::atomic::Ordering;

const MANIFEST: &str = "#EXTM3U\n#EXTINF:4,\nsegment.m4s\n#EXT-X-ENDLIST\n";
const ASSET: &[u8] = b"abcdefghijklmnop";

pub(super) async fn manifest() -> Response<Body> {
    Response::builder()
        .header(CONTENT_TYPE, "application/vnd.apple.mpegurl")
        .header(CONTENT_LENGTH, MANIFEST.len())
        .body(Body::from(MANIFEST))
        .expect("valid test fixture")
}

pub(super) async fn asset(State(state): State<OriginState>, headers: HeaderMap) -> Response<Body> {
    state
        .requests
        .lock()
        .expect("requests")
        .push(headers.clone());
    if state.hits.fetch_add(1, Ordering::SeqCst) == 0 {
        state.release.notified().await;
    }
    ranged_response(&headers)
}

fn ranged_response(headers: &HeaderMap) -> Response<Body> {
    let range = headers
        .get(RANGE)
        .expect("valid test fixture")
        .to_str()
        .expect("valid test fixture");
    let start: usize = range[6..]
        .split('-')
        .next()
        .expect("valid test fixture")
        .parse()
        .expect("valid test fixture");
    let end = (start + 4).min(ASSET.len());
    Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(
            CONTENT_RANGE,
            format!("bytes {start}-{}/{}", end - 1, ASSET.len()),
        )
        .header(CONTENT_LENGTH, end - start)
        .header(ETAG, "\"v1\"")
        .body(Body::from(ASSET[start..end].to_vec()))
        .expect("valid test fixture")
}
