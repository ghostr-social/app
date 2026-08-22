use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderMap, Response, Uri};
use axum::routing::get;
use axum::Router;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

pub type Requests = Arc<Mutex<Vec<(String, Option<String>, Option<String>)>>>;
pub const CHANGED_TOTAL_BYTES: usize = 400 * 1024;
pub const INIT_BYTES: usize = 300 * 1024;
pub const SHORT_INIT_BYTES: usize = 200 * 1024;

mod responses;
mod scenario;
use responses::{full, MANIFEST};
use scenario::{init, Change, FixtureState};

pub async fn serve() -> (String, Requests) {
    start(Change::Once).await
}

pub async fn serve_unstable() -> (String, Requests) {
    start(Change::Every).await
}

pub async fn serve_shortened() -> (String, Requests) {
    start(Change::Shorter).await
}

pub async fn serve_total_change() -> (String, Requests) {
    start(Change::Total).await
}

pub async fn serve_full_once() -> (String, Requests) {
    start(Change::FullOnce).await
}

pub async fn serve_full_unstable() -> (String, Requests) {
    start(Change::FullEvery).await
}

async fn start(change: Change) -> (String, Requests) {
    let requests = Requests::default();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = Router::new()
        .fallback(get(object))
        .with_state(FixtureState {
            requests: requests.clone(),
            change,
        });
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (format!("http://{address}/index.m3u8"), requests)
}

async fn object(State(state): State<FixtureState>, uri: Uri, headers: HeaderMap) -> Response<Body> {
    match uri.path() {
        "/index.m3u8" => full(MANIFEST, "root-v1"),
        "/init.mp4" => init(&headers, &state),
        "/segment.m4s" => full(b"segment", "segment-v1"),
        _ => Response::builder().status(404).body(Body::empty()).unwrap(),
    }
}
