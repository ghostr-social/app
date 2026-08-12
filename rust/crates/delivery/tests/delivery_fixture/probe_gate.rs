use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, Method, StatusCode};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;

#[derive(Clone)]
struct GateState {
    blocked: String,
    started: Arc<Semaphore>,
    release: Arc<Semaphore>,
}

pub struct ProbeGate {
    base: String,
    started: Arc<Semaphore>,
    release: Arc<Semaphore>,
}

impl ProbeGate {
    pub async fn serve(blocked: &str) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let started = Arc::new(Semaphore::new(0));
        let release = Arc::new(Semaphore::new(0));
        let state = GateState {
            blocked: blocked.to_owned(),
            started: Arc::clone(&started),
            release: Arc::clone(&release),
        };
        let app = Router::new().route("/{id}", get(media)).with_state(state);
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        Self {
            base: format!("http://{address}"),
            started,
            release,
        }
    }

    pub fn url(&self, id: &str) -> String {
        format!("{}/{id}", self.base)
    }

    pub async fn wait_blocked(&self) {
        self.started.acquire().await.unwrap().forget();
    }
}

impl Drop for ProbeGate {
    fn drop(&mut self) {
        self.release.add_permits(1);
    }
}

async fn media(
    Path(id): Path<String>,
    State(state): State<GateState>,
    method: Method,
    headers: HeaderMap,
) -> Response {
    if method == Method::HEAD {
        if id == state.blocked {
            state.started.add_permits(1);
            state.release.acquire().await.unwrap().forget();
        }
        return head_response();
    }
    range_response(&headers)
}

fn head_response() -> Response {
    Response::builder()
        .header(header::CONTENT_LENGTH, 64)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::ACCEPT_RANGES, "bytes")
        .body(Body::empty())
        .unwrap()
}

fn range_response(headers: &HeaderMap) -> Response {
    let range = headers
        .get(header::RANGE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("bytes=0-63");
    let (start, end) = range.trim_start_matches("bytes=").split_once('-').unwrap();
    let start = start.parse::<u64>().unwrap();
    let end = end.parse::<u64>().unwrap_or(63).min(63);
    Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(header::CONTENT_RANGE, format!("bytes {start}-{end}/64"))
        .body(Body::from(vec![1; (end - start + 1) as usize]))
        .unwrap()
}
