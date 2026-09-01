use axum::body::Body;
use axum::http::{header, Response};
use axum::routing::get;
use axum::Router;
use tokio::net::TcpListener;

pub(super) async fn start() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("HLS bind");
    let address = listener.local_addr().expect("HLS address");
    let router = Router::new()
        .route("/index.m3u8", get(manifest))
        .route("/segment.m4s", get(segment));
    tokio::spawn(async move { axum::serve(listener, router).await.expect("HLS serve") });
    format!("http://{address}/index.m3u8")
}

pub(super) fn origin(source: &str) -> String {
    source
        .strip_suffix("index.m3u8")
        .expect("fixture source suffix")
        .to_owned()
}

async fn manifest() -> Response<Body> {
    response(
        "application/vnd.apple.mpegurl",
        "#EXTM3U\n#EXTINF:4,\nsegment.m4s\n#EXT-X-ENDLIST\n",
        "manifest",
    )
}

async fn segment() -> Response<Body> {
    response("video/iso.segment", "fixture-segment", "segment")
}

fn response(content_type: &str, body: &str, tag: &str) -> Response<Body> {
    Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "max-age=60")
        .header(header::ETAG, format!("\"{tag}\""))
        .body(Body::from(body.to_owned()))
        .expect("HLS response")
}
