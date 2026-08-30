use axum::body::Body;
use axum::http::{header, Response};
use axum::routing::get;
use axum::Router;
use tokio::net::TcpListener;

pub async fn start() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind HLS origin");
    let address = listener.local_addr().expect("HLS origin address");
    let router = Router::new()
        .route("/index.m3u8", get(manifest))
        .route("/init.mp4", get(init))
        .route("/segment.m4s", get(segment));
    tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("serve HLS origin");
    });
    format!("http://{address}/index.m3u8")
}

pub fn origin(source: &str) -> String {
    source
        .strip_suffix("index.m3u8")
        .expect("fixture source suffix")
        .to_owned()
}

async fn manifest() -> Response<Body> {
    response(
        "application/vnd.apple.mpegurl",
        "#EXTM3U\n#EXT-X-MAP:URI=\"init.mp4\"\n#EXTINF:4,\nsegment.m4s\n#EXT-X-ENDLIST\n",
        "manifest",
    )
}

async fn init() -> Response<Body> {
    response("video/mp4", "fixture-init", "init")
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
        .expect("HLS fixture response")
}
