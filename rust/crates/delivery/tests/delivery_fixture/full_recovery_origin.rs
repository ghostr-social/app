use axum::body::Bytes;
use axum::http::Method;
use axum::routing::get;
use axum::Router;
use core::convert::Infallible;
use core::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

mod response;

pub const PROBE_BYTES: usize = 65_536;
pub const PARALLEL_BYTES: usize = 4_096;
pub const TRIAL_BYTES: usize = 900_000;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

pub(super) type BodySender = mpsc::Sender<Result<Bytes, Infallible>>;
pub(super) type OriginState = mpsc::Sender<ObservedRequest>;

pub struct RecoveryOrigin {
    authority: String,
    requests: mpsc::Receiver<ObservedRequest>,
}

pub struct ObservedRequest {
    pub method: Method,
    pub path: String,
    pub range: Option<String>,
    pub encoding: Option<String>,
    pub(super) body: Option<BodySender>,
}

impl RecoveryOrigin {
    pub async fn serve() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind origin");
        let authority = format!("http://{}", listener.local_addr().expect("origin address"));
        let (requests, observed) = mpsc::channel(8);
        let app = Router::new()
            .fallback(get(response::answer).head(response::answer))
            .with_state(requests);
        tokio::spawn(async move { axum::serve(listener, app).await.expect("serve origin") });
        Self {
            authority,
            requests: observed,
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{path}", self.authority)
    }

    pub async fn next(&mut self) -> ObservedRequest {
        self.requests.recv().await.expect("origin stays available")
    }

    pub async fn next_within(&mut self, label: &str) -> ObservedRequest {
        tokio::time::timeout(REQUEST_TIMEOUT, self.next())
            .await
            .unwrap_or_else(|_| panic!("missing {label}"))
    }

    pub async fn assert_quiet(&mut self) {
        if let Ok(request) = tokio::time::timeout(Duration::from_millis(50), self.next()).await {
            panic!(
                "unexpected {} {} range={:?}",
                request.method, request.path, request.range
            );
        }
    }
}

impl ObservedRequest {
    pub async fn send(&self, bytes: usize) {
        self.body
            .as_ref()
            .expect("GET body")
            .send(Ok(Bytes::from(vec![7; bytes])))
            .await
            .expect("open response");
    }

    pub async fn finish(mut self, bytes: usize) {
        self.send(bytes).await;
        self.body.take();
    }
}
