//! Range-capable fixture server that records every request as
//! `tag:METHOD:start-end` (or `tag:METHOD:full`) for order assertions.

use axum::body::Body;
use axum::extract::State;
use axum::http::{header, HeaderMap, Method, StatusCode};
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
    let state = Recorder {
        tag: tag.to_owned(),
        bytes: Arc::new(bytes),
        log,
    };
    let app = Router::new().route("/video.mp4", get(record)).with_state(state);
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind recorder");
    let address = listener.local_addr().expect("recorder address");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve recorder") });
    format!("http://{address}/video.mp4")
}

async fn record(State(state): State<Recorder>, method: Method, headers: HeaderMap) -> Response {
    let range = requested(&headers, state.bytes.len() as u64);
    let label = match range {
        Some((start, end)) => format!("{}:{method}:{start}-{end}", state.tag),
        None => format!("{}:{method}:full", state.tag),
    };
    state.log.lock().expect("hit log").push(label);
    match range {
        Some((start, end)) => partial(&state.bytes, start, end),
        None => full(&state.bytes),
    }
}

fn full(bytes: &[u8]) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::CONTENT_LENGTH, bytes.len())
        .header(header::ACCEPT_RANGES, "bytes")
        .body(Body::from(bytes.to_vec()))
        .expect("full response")
}

fn partial(bytes: &[u8], start: u64, end: u64) -> Response {
    let slice = bytes
        .get(start as usize..(end + 1) as usize)
        .unwrap_or(&[])
        .to_vec();
    Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::CONTENT_LENGTH, slice.len())
        .header(
            header::CONTENT_RANGE,
            format!("bytes {start}-{end}/{}", bytes.len()),
        )
        .body(Body::from(slice))
        .expect("partial response")
}

fn requested(headers: &HeaderMap, len: u64) -> Option<(u64, u64)> {
    let value = headers.get(header::RANGE)?.to_str().ok()?;
    let (start, end) = value.strip_prefix("bytes=")?.split_once('-')?;
    let start = start.parse().ok()?;
    let end: u64 = end.parse().unwrap_or(len - 1);
    Some((start, end.min(len - 1)))
}
