//! Range-capable fixture server that records every request as
//! `tag:METHOD:start-end` (or `tag:METHOD:full`) for order assertions.

mod request;
mod response;

use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

pub type HitLog = Arc<Mutex<Vec<String>>>;

pub fn hit_log() -> HitLog {
    Arc::new(Mutex::new(Vec::new()))
}

pub fn media_body() -> Vec<u8> {
    b"0123456789abcdef".to_vec()
}

pub fn hits(log: &HitLog) -> Vec<String> {
    log.lock().expect("hit log").clone()
}

#[derive(Clone)]
struct Recorder {
    tag: String,
    bytes: Arc<Vec<u8>>,
    log: HitLog,
}

pub async fn serve_recording(tag: &str, bytes: Vec<u8>, log: HitLog) -> String {
    serve(
        Router::new()
            .route("/video.mp4", get(record))
            .with_state(recorder(tag, bytes, log)),
    )
    .await
}

fn recorder(tag: &str, bytes: Vec<u8>, log: HitLog) -> Recorder {
    Recorder {
        tag: tag.to_owned(),
        bytes: Arc::new(bytes),
        log,
    }
}

async fn serve(app: Router) -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind recorder");
    let address = listener.local_addr().expect("recorder address");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve recorder") });
    format!("http://{address}/video.mp4")
}

/// Records every attempt like [`serve_recording`] but answers `404`,
/// the shape of a source that is gone for good.
pub async fn serve_rejecting(tag: &str, log: HitLog) -> String {
    let state = recorder(tag, Vec::new(), log);
    serve(
        Router::new()
            .route("/video.mp4", get(reject))
            .with_state(state),
    )
    .await
}

async fn reject(State(state): State<Recorder>, method: Method, headers: HeaderMap) -> Response {
    request::note(&state, &method, &headers);
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::empty())
        .expect("rejected response")
}

async fn record(State(state): State<Recorder>, method: Method, headers: HeaderMap) -> Response {
    match request::note(&state, &method, &headers) {
        Some((start, end)) => response::partial(&state.bytes, start, end),
        None => response::full(&state.bytes),
    }
}
