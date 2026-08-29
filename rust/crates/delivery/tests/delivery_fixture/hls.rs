use axum::body::Body;
use axum::extract::State;
use axum::http::{header, Response};
use axum::routing::get;
use axum::Router;
use tokio::net::TcpListener;

mod gate;

pub use gate::HlsGate;

pub async fn serve(gate: HlsGate) -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("valid test fixture");
    let address = listener.local_addr().expect("valid test fixture");
    let app = Router::new()
        .route("/index.m3u8", get(manifest))
        .route("/child.m3u8", get(child))
        .route("/init.mp4", get(init))
        .route("/segment.m4s", get(segment))
        .with_state(gate);
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("valid test fixture");
    });
    format!("http://{address}/index.m3u8")
}

async fn manifest(State(gate): State<HlsGate>) -> Response<Body> {
    gate.hit("root").await;
    response(
        "application/vnd.apple.mpegurl",
        b"#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=1000000\nchild.m3u8\n",
    )
}

async fn child(State(gate): State<HlsGate>) -> Response<Body> {
    gate.hit("child").await;
    response(
        "application/vnd.apple.mpegurl",
        b"#EXTM3U\n#EXT-X-TARGETDURATION:4\n#EXT-X-MAP:URI=\"init.mp4\"\n\
          #EXTINF:4,\nsegment.m4s\n#EXT-X-ENDLIST\n",
    )
}

async fn init(State(gate): State<HlsGate>) -> Response<Body> {
    gate.hit("init").await;
    response("video/mp4", b"init")
}

async fn segment(State(gate): State<HlsGate>) -> Response<Body> {
    gate.hit("segment").await;
    response("video/iso.segment", b"segment")
}

fn response(content_type: &'static str, bytes: &'static [u8]) -> Response<Body> {
    Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .body(Body::from(bytes))
        .expect("valid test fixture")
}
