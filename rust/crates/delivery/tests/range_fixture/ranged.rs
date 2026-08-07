//! Axum fixture endpoints with HTTP range semantics for downloader and
//! probe tests: a range-capable server and a range-ignoring server.

use axum::body::Body;
use axum::extract::State;
use axum::http::response::Builder;
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::Response;
use axum::routing::{get, MethodRouter};
use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct Media(pub Arc<Vec<u8>>);

pub async fn serve_ranged(bytes: Vec<u8>) -> String {
    serve(app(get(ranged), bytes)).await
}

pub async fn serve_range_blind(bytes: Vec<u8>) -> String {
    serve(app(get(blind), bytes)).await
}

pub fn app(route: MethodRouter<Media>, bytes: Vec<u8>) -> Router {
    Router::new()
        .route("/video.mp4", route)
        .with_state(Media(Arc::new(bytes)))
}

pub async fn serve(app: Router) -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind fixture");
    let address = listener.local_addr().expect("fixture address");
    tokio::spawn(async move { axum::serve(listener, app).await.expect("serve fixture") });
    format!("http://{address}/video.mp4")
}

pub async fn ranged(State(media): State<Media>, headers: HeaderMap) -> Response {
    match requested_range(&headers, media.0.len() as u64) {
        Some((start, end)) => partial(&media.0, start, end),
        None => full(&media.0),
    }
}

async fn blind(State(media): State<Media>) -> Response {
    builder(StatusCode::OK, media.0.len())
        .body(Body::from(media.0.to_vec()))
        .expect("blind response")
}

fn full(media: &[u8]) -> Response {
    builder(StatusCode::OK, media.len())
        .header(header::ACCEPT_RANGES, "bytes")
        .body(Body::from(media.to_vec()))
        .expect("full response")
}

fn partial(media: &[u8], start: u64, end: u64) -> Response {
    let slice = media[start as usize..=end as usize].to_vec();
    builder(StatusCode::PARTIAL_CONTENT, slice.len())
        .header(
            header::CONTENT_RANGE,
            format!("bytes {start}-{end}/{}", media.len()),
        )
        .body(Body::from(slice))
        .expect("partial response")
}

fn builder(status: StatusCode, length: usize) -> Builder {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "video/mp4")
        .header(header::CONTENT_LENGTH, length)
}

fn requested_range(headers: &HeaderMap, len: u64) -> Option<(u64, u64)> {
    let value = headers.get(header::RANGE)?.to_str().ok()?;
    let (start, end) = value.strip_prefix("bytes=")?.split_once('-')?;
    let start = start.parse().ok()?;
    let end: u64 = end.parse().unwrap_or(len - 1);
    Some((start, end.min(len - 1)))
}
