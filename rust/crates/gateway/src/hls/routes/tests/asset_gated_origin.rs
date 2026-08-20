use axum::http::header::IF_RANGE;
use axum::http::HeaderMap;
use axum::routing::get;
use axum::Router;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::Notify;

mod response;
use response::{asset, manifest};

#[derive(Clone)]
struct OriginState {
    hits: Arc<AtomicUsize>,
    release: Arc<Notify>,
    requests: Arc<Mutex<Vec<HeaderMap>>>,
}

pub(super) struct GatedOrigin {
    state: OriginState,
    task: tokio::task::JoinHandle<()>,
}

impl GatedOrigin {
    pub async fn start() -> (String, Self) {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("listener");
        let state = OriginState {
            hits: Arc::new(AtomicUsize::new(0)),
            release: Arc::new(Notify::new()),
            requests: Arc::new(Mutex::new(Vec::new())),
        };
        let app = Router::new()
            .route("/index.m3u8", get(manifest))
            .route("/segment.m4s", get(asset))
            .with_state(state.clone());
        let address = listener.local_addr().expect("origin address");
        let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        (format!("http://{address}/index.m3u8"), Self { state, task })
    }

    pub async fn wait_hits(&self, expected: usize) {
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            while self.hits() < expected {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("origin asset request");
    }

    pub fn hits(&self) -> usize {
        self.state.hits.load(Ordering::SeqCst)
    }

    pub fn release_first(&self) {
        self.state.release.notify_one();
    }

    pub fn if_ranges(&self) -> Vec<Option<String>> {
        self.state
            .requests
            .lock()
            .expect("requests")
            .iter()
            .map(|headers| text(headers, IF_RANGE))
            .collect()
    }
}

impl Drop for GatedOrigin {
    fn drop(&mut self) {
        self.task.abort();
    }
}

fn text(headers: &HeaderMap, name: axum::http::HeaderName) -> Option<String> {
    headers.get(name)?.to_str().ok().map(str::to_owned)
}
