use axum::body::Body;
use axum::extract::State;
use axum::http::{header, Response};
use axum::routing::get;
use axum::Router;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;

#[derive(Clone)]
pub struct HlsGate {
    pub started: Arc<Semaphore>,
    pub release: Arc<Semaphore>,
    hits: Arc<Mutex<Vec<&'static str>>>,
    blocked: &'static str,
}

impl HlsGate {
    pub fn new() -> Self {
        Self::blocking("root")
    }

    pub fn blocking(blocked: &'static str) -> Self {
        Self {
            started: Arc::new(Semaphore::new(0)),
            release: Arc::new(Semaphore::new(0)),
            hits: Arc::new(Mutex::new(Vec::new())),
            blocked,
        }
    }

    pub fn hits(&self) -> Vec<&'static str> {
        self.hits.lock().expect("valid test fixture").clone()
    }

    async fn hit(&self, path: &'static str) {
        self.hits.lock().expect("valid test fixture").push(path);
        if path == self.blocked {
            self.started.add_permits(1);
            self.release
                .acquire()
                .await
                .expect("valid test fixture")
                .forget();
        }
    }
}

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
