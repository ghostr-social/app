use axum::body::Body;
use axum::extract::State;
use axum::http::{header, Response};
use axum::routing::get;
use axum::Router;
use core::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct PreparedHlsOrigin {
    version: Arc<RwLock<&'static str>>,
    hits: Arc<AtomicUsize>,
}

impl PreparedHlsOrigin {
    pub async fn start(version: &'static str) -> (Self, String) {
        let origin = Self {
            version: Arc::new(RwLock::new(version)),
            hits: Arc::new(AtomicUsize::new(0)),
        };
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("valid test fixture");
        let address = listener.local_addr().expect("valid test fixture");
        let app = Router::new()
            .route("/index.m3u8", get(manifest))
            .route("/init.mp4", get(init))
            .route("/segment.m4s", get(segment))
            .with_state(origin.clone());
        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("valid test fixture");
        });
        (origin, format!("http://{address}/index.m3u8"))
    }

    pub fn hits(&self) -> usize {
        self.hits.load(Ordering::SeqCst)
    }

    pub fn set_version(&self, version: &'static str) {
        *self.version.write().expect("version state") = version;
    }

    fn version(&self) -> &'static str {
        *self.version.read().expect("version state")
    }
}

async fn manifest(State(origin): State<PreparedHlsOrigin>) -> Response<Body> {
    response(
        &origin,
        "root",
        "application/vnd.apple.mpegurl",
        "#EXTM3U\n#EXT-X-MAP:URI=\"init.mp4\"\n#EXTINF:4,\nsegment.m4s\n#EXT-X-ENDLIST\n"
            .to_owned(),
    )
}

async fn init(State(origin): State<PreparedHlsOrigin>) -> Response<Body> {
    response(
        &origin,
        "init",
        "video/mp4",
        format!("{}-init", origin.version()),
    )
}

async fn segment(State(origin): State<PreparedHlsOrigin>) -> Response<Body> {
    response(
        &origin,
        "segment",
        "video/iso.segment",
        format!("{}-segment", origin.version()),
    )
}

fn response(
    origin: &PreparedHlsOrigin,
    name: &str,
    content_type: &str,
    body: String,
) -> Response<Body> {
    origin.hits.fetch_add(1, Ordering::SeqCst);
    let version = origin.version();
    Response::builder()
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CACHE_CONTROL, "max-age=60")
        .header(header::ETAG, format!("\"{version}-{name}\""))
        .body(Body::from(body))
        .expect("valid test fixture")
}
