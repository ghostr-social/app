use super::HlsOrigin;
use axum::body::Body;
use axum::extract::State;
use axum::http::{header, Response};
use std::sync::atomic::Ordering;

pub(super) async fn manifest(state: State<HlsOrigin>) -> Response<Body> {
    if state.master {
        return response(
            &state,
            "root",
            "application/vnd.apple.mpegurl",
            b"#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1000000\nchild.m3u8\n",
        );
    }
    response(
        &state,
        "root",
        "application/vnd.apple.mpegurl",
        b"#EXTM3U\n#EXT-X-MAP:URI=\"init.mp4\"\n#EXTINF:4,\nsegment.m4s\n#EXT-X-ENDLIST\n",
    )
}

pub(super) async fn child(state: State<HlsOrigin>) -> Response<Body> {
    response(
        &state,
        "child",
        "application/vnd.apple.mpegurl",
        b"#EXTM3U\n#EXT-X-MAP:URI=\"init.mp4\"\n#EXTINF:4,\nsegment.m4s\n#EXT-X-ENDLIST\n",
    )
}

pub(super) async fn init(state: State<HlsOrigin>) -> Response<Body> {
    response(&state, "init", "video/mp4", b"init")
}

pub(super) async fn segment(state: State<HlsOrigin>) -> Response<Body> {
    response(&state, "segment", "video/iso.segment", b"segment")
}

fn response(
    state: &HlsOrigin,
    path: &'static str,
    content_type: &'static str,
    body: &'static [u8],
) -> Response<Body> {
    state.hits.fetch_add(1, Ordering::SeqCst);
    state.paths.lock().unwrap().push(path);
    let mut response = Response::builder().header(header::CONTENT_TYPE, content_type);
    if state.cacheable {
        response = response
            .header(header::CACHE_CONTROL, "max-age=60")
            .header(header::ETAG, format!("\"fixture-hls-{path}\""));
    }
    response.body(Body::from(body)).unwrap()
}
