use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::net::TcpListener;
use tokio::sync::Notify;

mod responses;

#[derive(Clone, Copy, Debug)]
pub struct HlsHit {
    pub path: &'static str,
    pub at: Instant,
}

#[derive(Clone)]
pub struct HlsScript {
    shared: Arc<Shared>,
}

struct Shared {
    failure_path: &'static str,
    statuses: Mutex<VecDeque<StatusCode>>,
    hits: Mutex<Vec<HlsHit>>,
    changed: Notify,
}

impl HlsScript {
    pub fn new(failure_path: &'static str, statuses: impl IntoIterator<Item = StatusCode>) -> Self {
        Self {
            shared: Arc::new(Shared {
                failure_path,
                statuses: Mutex::new(statuses.into_iter().collect()),
                hits: Mutex::new(Vec::new()),
                changed: Notify::new(),
            }),
        }
    }

    pub fn paths(&self) -> Vec<&'static str> {
        self.hits().into_iter().map(|hit| hit.path).collect()
    }

    pub fn hits(&self) -> Vec<HlsHit> {
        self.shared.hits.lock().unwrap().clone()
    }

    pub async fn wait_for_hits(&self, count: usize) {
        tokio::time::timeout(std::time::Duration::from_secs(2), async {
            loop {
                let changed = self.shared.changed.notified();
                tokio::pin!(changed);
                changed.as_mut().enable();
                if self.hits().len() >= count {
                    return;
                }
                changed.await;
            }
        })
        .await
        .expect("scripted HLS request count");
    }

    async fn record(&self, path: &'static str) -> Option<StatusCode> {
        self.shared.hits.lock().unwrap().push(HlsHit {
            path,
            at: Instant::now(),
        });
        self.shared.changed.notify_waiters();
        (path == self.shared.failure_path)
            .then(|| self.shared.statuses.lock().unwrap().pop_front())
            .flatten()
    }
}

pub async fn serve(script: HlsScript) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = Router::new()
        .route("/index.m3u8", get(responses::root))
        .route("/child.m3u8", get(responses::child))
        .route("/init.mp4", get(responses::init))
        .route("/segment.m4s", get(responses::segment))
        .with_state(script);
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{address}/index.m3u8")
}
