use axum::routing::get;
use axum::Router;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::net::TcpListener;

mod responses;
use responses::{child, init, manifest, segment};

#[derive(Clone)]
pub struct HlsOrigin {
    hits: Arc<AtomicUsize>,
    paths: Arc<Mutex<Vec<&'static str>>>,
    master: bool,
    cacheable: bool,
}

impl HlsOrigin {
    pub async fn start() -> (Self, String) {
        Self::start_with(false, false).await
    }

    pub async fn start_master() -> (Self, String) {
        Self::start_with(true, false).await
    }

    pub async fn start_cacheable() -> (Self, String) {
        Self::start_with(false, true).await
    }

    pub async fn start_cacheable_master() -> (Self, String) {
        Self::start_with(true, true).await
    }

    async fn start_with(master: bool, cacheable: bool) -> (Self, String) {
        let origin = Self {
            hits: Arc::new(AtomicUsize::new(0)),
            paths: Arc::new(Mutex::new(Vec::new())),
            master,
            cacheable,
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
