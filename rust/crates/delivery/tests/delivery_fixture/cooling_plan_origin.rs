use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;

#[derive(Clone)]
struct OriginState {
    failures: Arc<AtomicUsize>,
    useful: Arc<AtomicUsize>,
    started: Arc<Semaphore>,
    release: Arc<Semaphore>,
}

pub struct CoolingPlanOrigin {
    base: String,
    state: OriginState,
}

impl CoolingPlanOrigin {
    pub async fn serve() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let state = OriginState {
            failures: Arc::new(AtomicUsize::new(0)),
            useful: Arc::new(AtomicUsize::new(0)),
            started: Arc::new(Semaphore::new(0)),
            release: Arc::new(Semaphore::new(0)),
        };
        let app = Router::new()
            .route("/{kind}", get(media))
            .with_state(state.clone());
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        Self {
            base: format!("http://{address}"),
            state,
        }
    }

    pub fn url(&self, kind: &str) -> String {
        format!("{}/{kind}", self.base)
    }

    pub fn failures(&self) -> usize {
        self.state.failures.load(Ordering::SeqCst)
    }

    pub fn useful_requests(&self) -> usize {
        self.state.useful.load(Ordering::SeqCst)
    }

    pub async fn wait_useful(&self) {
        self.state.started.acquire().await.unwrap().forget();
    }

    pub fn release(&self) {
        self.state.release.add_permits(4);
    }
}

async fn media(
    Path(kind): Path<String>,
    State(state): State<OriginState>,
    headers: HeaderMap,
) -> Response {
    if kind == "cooling" {
        state.failures.fetch_add(1, Ordering::SeqCst);
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap();
    }
    state.useful.fetch_add(1, Ordering::SeqCst);
    state.started.add_permits(1);
    state.release.acquire().await.unwrap().forget();
    ranged_response(&headers)
}

fn ranged_response(headers: &HeaderMap) -> Response {
    let value = headers[header::RANGE].to_str().unwrap();
    let (start, end) = value.trim_start_matches("bytes=").split_once('-').unwrap();
    let start: u64 = start.parse().unwrap();
    let end = end.parse::<u64>().unwrap_or(63).min(63);
    Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::ETAG, "\"fixture-cooling\"")
        .header(header::CONTENT_RANGE, format!("bytes {start}-{end}/64"))
        .body(Body::from(vec![7; (end - start + 1) as usize]))
        .unwrap()
}
