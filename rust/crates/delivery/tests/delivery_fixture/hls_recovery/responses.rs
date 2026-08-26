use super::HlsScript;
use axum::body::Body;
use axum::extract::State;
use axum::http::{header, Response, StatusCode};

pub(super) async fn root(State(script): State<HlsScript>) -> Response<Body> {
    response(
        &script,
        "root",
        "application/vnd.apple.mpegurl",
        b"#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1000000\nchild.m3u8\n",
    )
    .await
}

pub(super) async fn child(State(script): State<HlsScript>) -> Response<Body> {
    response(
        &script,
        "child",
        "application/vnd.apple.mpegurl",
        b"#EXTM3U\n#EXT-X-TARGETDURATION:4\n#EXT-X-MAP:URI=\"init.mp4\"\n\
          #EXTINF:4,\nsegment.m4s\n#EXT-X-ENDLIST\n",
    )
    .await
}

pub(super) async fn init(State(script): State<HlsScript>) -> Response<Body> {
    response(&script, "init", "video/mp4", b"init").await
}

pub(super) async fn segment(State(script): State<HlsScript>) -> Response<Body> {
    response(&script, "segment", "video/iso.segment", b"segment").await
}

fn response(
    script: &HlsScript,
    path: &'static str,
    content_type: &'static str,
    body: &'static [u8],
) -> impl core::future::Future<Output = Response<Body>> {
    let status = script.record(path).unwrap_or(StatusCode::OK);
    core::future::ready(
        Response::builder()
            .status(status)
            .header(header::CONTENT_TYPE, content_type)
            .body(Body::from(body))
            .expect("valid test fixture"),
    )
}
