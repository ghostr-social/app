use axum::body::Body;
use axum::http::{header, Response};
use axum::routing::get;
use axum::Router;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct HlsOrigin {
    hits: Arc<AtomicUsize>,
    paths: Arc<Mutex<Vec<&'static str>>>,
    master: bool,
}

impl HlsOrigin {
    pub async fn start() -> (Self, String) {
        Self::start_with(false).await
    }

    pub async fn start_master() -> (Self, String) {
        Self::start_with(true).await
    }

    async fn start_with(master: bool) -> (Self, String) {
        let origin = Self {
            hits: Arc::new(AtomicUsize::new(0)),
            paths: Arc::new(Mutex::new(Vec::new())),
            master,
        };
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let app = Router::new()
            .route("/index.m3u8", get(manifest))
            .route("/child.m3u8", get(child))
            .route("/init.mp4", get(init))
            .route("/segment.m4s", get(segment))
            .with_state(origin.clone());
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        (origin, format!("http://{address}/index.m3u8"))
    }

    pub fn hits(&self) -> usize {
        self.hits.load(Ordering::SeqCst)
    }

    pub fn paths(&self) -> Vec<&'static str> {
        self.paths.lock().unwrap().clone()
    }
}

async fn manifest(state: axum::extract::State<HlsOrigin>) -> Response<Body> {
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

async fn child(state: axum::extract::State<HlsOrigin>) -> Response<Body> {
    response(
        &state,
        "child",
        "application/vnd.apple.mpegurl",
        b"#EXTM3U\n#EXT-X-MAP:URI=\"init.mp4\"\n#EXTINF:4,\nsegment.m4s\n#EXT-X-ENDLIST\n",
    )
}

async fn init(state: axum::extract::State<HlsOrigin>) -> Response<Body> {
    response(&state, "init", "video/mp4", b"init")
}

async fn segment(state: axum::extract::State<HlsOrigin>) -> Response<Body> {
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
    Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .body(Body::from(body))
        .unwrap()
}
