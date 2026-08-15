use axum::body::Body;
use axum::http::{header, Response};
use axum::routing::get;
use axum::Router;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct HlsOrigin {
    hits: Arc<AtomicUsize>,
}

impl HlsOrigin {
    pub async fn start() -> (Self, String) {
        let origin = Self {
            hits: Arc::new(AtomicUsize::new(0)),
        };
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let app = Router::new()
            .route("/index.m3u8", get(manifest))
            .route("/init.mp4", get(init))
            .route("/segment.m4s", get(segment))
            .with_state(origin.clone());
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        (origin, format!("http://{address}/index.m3u8"))
    }

    pub fn hits(&self) -> usize {
        self.hits.load(Ordering::SeqCst)
    }
}

async fn manifest(state: axum::extract::State<HlsOrigin>) -> Response<Body> {
    response(
        &state,
        "application/vnd.apple.mpegurl",
        b"#EXTM3U\n#EXT-X-MAP:URI=\"init.mp4\"\n#EXTINF:4,\nsegment.m4s\n#EXT-X-ENDLIST\n",
    )
}

async fn init(state: axum::extract::State<HlsOrigin>) -> Response<Body> {
    response(&state, "video/mp4", b"init")
}

async fn segment(state: axum::extract::State<HlsOrigin>) -> Response<Body> {
    response(&state, "video/iso.segment", b"segment")
}

fn response(state: &HlsOrigin, content_type: &'static str, body: &'static [u8]) -> Response<Body> {
    state.hits.fetch_add(1, Ordering::SeqCst);
    Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .body(Body::from(body))
        .unwrap()
}
