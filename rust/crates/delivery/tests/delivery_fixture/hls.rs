use axum::body::Body;
use axum::extract::State;
use axum::http::{header, Response};
use axum::routing::get;
use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;

#[derive(Clone)]
pub struct HlsGate {
    pub started: Arc<Semaphore>,
    pub release: Arc<Semaphore>,
}

impl HlsGate {
    pub fn new() -> Self {
        Self {
            started: Arc::new(Semaphore::new(0)),
            release: Arc::new(Semaphore::new(0)),
        }
    }
}

pub async fn serve(gate: HlsGate) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = Router::new()
        .route("/index.m3u8", get(manifest))
        .route("/init.mp4", get(init))
        .route("/segment.m4s", get(segment))
        .with_state(gate);
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{address}/index.m3u8")
}

async fn manifest(State(gate): State<HlsGate>) -> Response<Body> {
    gate.started.add_permits(1);
    gate.release.acquire().await.unwrap().forget();
    response(
        "application/vnd.apple.mpegurl",
        b"#EXTM3U\n#EXT-X-TARGETDURATION:4\n#EXT-X-MAP:URI=\"init.mp4\"\n\
          #EXTINF:4,\nsegment.m4s\n#EXT-X-ENDLIST\n",
    )
}

async fn init() -> Response<Body> {
    response("video/mp4", b"init")
}

async fn segment() -> Response<Body> {
    response("video/iso.segment", b"segment")
}

fn response(content_type: &'static str, bytes: &'static [u8]) -> Response<Body> {
    Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .body(Body::from(bytes))
        .unwrap()
}
